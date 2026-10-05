//! Tauri-Backend von Hows.
//!
//! Tray, globale Hotkeys für Aufnahme und Pause (konfigurierbar),
//! Start/Stop/Pause und die Editor-Shell. Der Capture-Kern läuft
//! in-process in einem dedizierten Thread (Spec §3).

mod capture_window;
mod commands;
mod error;
mod open_guide;
mod recorder;
mod settings;
mod tray;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use capture_window::CaptureWindow;
use commands::{load_guide_from_path, show_main_window, AppState, HotkeySlot};
use error::{CommandError, ErrorCode};
use open_guide::steps_path_from_args;
use recorder::Command;

/// Registriert einen Hotkey (validiert den String).
pub(crate) fn register_hotkey(app: &AppHandle, hotkey: &str) -> Result<(), CommandError> {
    let shortcut: Shortcut = hotkey
        .parse()
        .map_err(CommandError::with(ErrorCode::HotkeyInvalid))?;
    app.global_shortcut()
        .register(shortcut)
        .map_err(CommandError::with(ErrorCode::HotkeyTaken))
}

/// Entfernt eine Hotkey-Registrierung (Fehler sind unkritisch).
pub(crate) fn unregister_hotkey(app: &AppHandle, hotkey: &str) {
    if let Ok(shortcut) = hotkey.parse::<Shortcut>() {
        app.global_shortcut().unregister(shortcut).ok();
    }
}

/// Zweiter Start: bestehendes Fenster nach vorn, optional `.steps` öffnen.
fn on_second_instance(app: &AppHandle, argv: Vec<String>) {
    // Während der Aufnahme nur die Pill zeigen. Die Datei ersetzt die Session nicht.
    if app
        .try_state::<CaptureWindow>()
        .is_some_and(|capture| capture.active())
    {
        capture_window::reveal(app);
        return;
    }
    show_main_window(app.clone());
    let Some(path) = steps_path_from_args(argv) else {
        return;
    };
    // Das Frontend fragt, bevor eine ungespeicherte Anleitung ersetzt wird.
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut pending) = state.pending_open.lock() {
            *pending = Some(path.display().to_string());
        }
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.emit("guide://open-requested", ());
    }
}

/// Startet die Tauri-Anwendung.
///
/// # Panics
///
/// Bricht ab, wenn die Tauri-Runtime nicht gestartet werden kann.
pub fn run() {
    tauri::Builder::default()
        // Muss zuerst stehen: sonst startet ein zweiter Prozess Hotkeys/Tray mit.
        .plugin(tauri_plugin_single_instance::init(|app, argv, _cwd| {
            on_second_instance(app, argv);
        }))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    let Some(state) = app.try_state::<AppState>() else {
                        return;
                    };
                    // Welcher Hotkey ausgelöst hat, entscheidet über das
                    // Kommando: Aufnahme = Start/Stop, Pause = Pause/Resume.
                    let pause_hotkey = state
                        .settings
                        .lock()
                        .map(|settings| settings.pause_hotkey.clone())
                        .unwrap_or_default();
                    let is_pause = pause_hotkey
                        .parse::<Shortcut>()
                        .is_ok_and(|pause| pause == *shortcut);
                    if is_pause {
                        state.recorder.send(Command::TogglePause).ok();
                    } else {
                        state.recorder.send(Command::Toggle).ok();
                    }
                })
                .build(),
        )
        .setup(|app| {
            let config_dir = app.path().app_config_dir()?;
            let app_settings = settings::load(&config_dir);

            let locale = app_settings.locale();
            let guide_slot = Arc::new(Mutex::new(None));
            let recorder = recorder::spawn(
                app.handle().clone(),
                guide_slot.clone(),
                recorder::RecorderConfig::from_settings(&app_settings),
            );

            let mut unregistered_hotkeys = Vec::new();
            for (slot, hotkey) in [
                (HotkeySlot::Toggle, &app_settings.hotkey),
                (HotkeySlot::Pause, &app_settings.pause_hotkey),
            ] {
                if let Err(error) = register_hotkey(app.handle(), hotkey) {
                    eprintln!("Hotkey nicht registrierbar: {error} ({hotkey})");
                    unregistered_hotkeys.push(slot.name().to_owned());
                }
            }

            app.manage(capture_window::CaptureWindow::default());
            app.manage(AppState {
                recorder,
                guide: guide_slot,
                settings: Mutex::new(app_settings),
                pending_open: Mutex::new(None),
                unregistered_hotkeys: Mutex::new(unregistered_hotkeys),
            });

            // Dateizuordnung oder Kommandozeile. `app.exe path.steps` öffnet den Editor, nicht Bereit.
            if let Some(path) = steps_path_from_args(std::env::args_os()) {
                let state = app.state::<AppState>();
                if let Err(error) = load_guide_from_path(app.handle(), &state, &path) {
                    eprintln!("argv open failed ({error})");
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.emit("guide://open-error", error);
                    }
                }
            }

            let tray_menu = tray::build(app, locale)?;
            app.manage(tray_menu);
            Ok(())
        })
        // Alt+F4 beendet. Jedes andere Schließen während der Aufnahme pausiert
        // und versteckt die Leiste. Sonst bleibt Schließen ein Verstecken.
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let app = window.app_handle();
                match choose_close(alt_f4_down(), capture_active(app)) {
                    CloseChoice::Quit => {
                        let _ = app.emit("app://quit-requested", ());
                    }
                    CloseChoice::PauseAndHide => {
                        pause_for_hide(app);
                        window.hide().ok();
                    }
                    CloseChoice::Hide => {
                        window.hide().ok();
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::recorder_command,
            commands::show_main_window,
            commands::quit_app,
            commands::take_pending_open,
            commands::get_guide,
            commands::get_step_image,
            commands::set_guide_title,
            commands::set_step_text,
            commands::set_step_overlays,
            commands::set_step_crop,
            commands::delete_step,
            commands::move_step,
            commands::save_guide,
            commands::open_guide,
            commands::get_recent_guides,
            commands::get_guide_thumbnail,
            commands::export_html,
            commands::export_pdf,
            commands::export_markdown,
            commands::export_json,
            commands::get_settings,
            commands::get_settings_defaults,
            commands::change_setting,
            commands::reset_advanced_settings,
            commands::set_hotkey,
            commands::set_pause_hotkey,
            commands::set_language,
            commands::set_export_format,
            commands::set_open_after_export,
            commands::set_theme,
            commands::set_default_export_folder,
            commands::set_guides_folder,
            commands::pick_folder,
            commands::reveal_path,
            capture_window::fit_capture_window,
            capture_window::present_for_dialog,
            capture_window::restore_capture_pill,
        ])
        .run(tauri::generate_context!())
        .expect("Fehler beim Starten der Tauri-Anwendung");
}

/// Was Schließen mit dem Hauptfenster macht.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CloseChoice {
    /// Alt+F4: Hows beenden. Die Rückfrage läuft im Frontend.
    Quit,
    /// Taskleiste und jedes andere Schließen während der Aufnahme.
    PauseAndHide,
    /// Schließen außerhalb der Aufnahme versteckt das Fenster.
    Hide,
}

fn choose_close(alt_f4: bool, capture_active: bool) -> CloseChoice {
    if alt_f4 {
        CloseChoice::Quit
    } else if capture_active {
        CloseChoice::PauseAndHide
    } else {
        CloseChoice::Hide
    }
}

fn capture_active(app: &AppHandle) -> bool {
    app.try_state::<CaptureWindow>()
        .is_some_and(|capture| capture.active())
}

/// Pause ist durch, bevor die Leiste verschwindet. Schon pausiert bleibt pausiert.
fn pause_for_hide(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let (done_tx, done_rx) = std::sync::mpsc::channel();
    if state.recorder.send(Command::PauseSettled(done_tx)).is_ok() {
        let _ = done_rx.recv_timeout(Duration::from_secs(1));
    }
}

/// Alt und F4 sind im Moment des Schließens gedrückt.
fn alt_f4_down() -> bool {
    #[cfg(windows)]
    {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            GetAsyncKeyState, GetKeyState, VK_F4, VK_MENU,
        };
        fn down(vk: u16) -> bool {
            let code = i32::from(vk);
            // Das hohe Bit heißt gedrückt: als i16 also negativ. GetKeyState
            // hängt an der Meldung, GetAsyncKeyState am aktuellen Zustand.
            // SAFETY: beide Funktionen lesen nur den Tastenstatus.
            unsafe { GetAsyncKeyState(code) < 0 || GetKeyState(code) < 0 }
        }
        down(VK_MENU.0) && down(VK_F4.0)
    }
    #[cfg(not(windows))]
    {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::{choose_close, CloseChoice};

    #[test]
    fn alt_f4_beendet_auch_waehrend_der_aufnahme() {
        assert_eq!(choose_close(true, true), CloseChoice::Quit);
        assert_eq!(choose_close(true, false), CloseChoice::Quit);
    }

    #[test]
    fn anderes_schliessen_pausiert_nur_waehrend_der_aufnahme() {
        assert_eq!(choose_close(false, true), CloseChoice::PauseAndHide);
        assert_eq!(choose_close(false, false), CloseChoice::Hide);
    }
}
