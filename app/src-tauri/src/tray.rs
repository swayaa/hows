//! Tray-Symbol mit Menü. Die Einträge bleiben erhalten, damit ein
//! Sprachwechsel nur ihre Texte neu setzt.

use steps_i18n::{Key, Locale};
use tauri::menu::{IsMenuItem, Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{Emitter, Manager, Wry};

use crate::commands::{show_main_window, AppState};
use crate::recorder::Command;

const ENTRIES: [(&str, Key); 4] = [
    ("toggle", Key::TrayToggle),
    ("pause", Key::TrayPause),
    ("show", Key::TrayShow),
    ("quit", Key::TrayQuit),
];

/// Menüeinträge des Tray-Symbols samt Textschlüssel.
pub struct TrayMenu(Vec<(MenuItem<Wry>, Key)>);

impl TrayMenu {
    /// Setzt alle Menütexte in `locale`.
    pub fn apply(&self, locale: Locale) {
        for (item, key) in &self.0 {
            item.set_text(locale.text(*key)).ok();
        }
    }
}

/// Baut das Tray-Symbol und liefert die Menüeinträge für spätere
/// Sprachwechsel.
pub fn build(app: &tauri::App, locale: Locale) -> tauri::Result<TrayMenu> {
    let items = ENTRIES
        .iter()
        .map(|&(id, key)| {
            MenuItem::with_id(app, id, locale.text(key), true, None::<&str>).map(|item| (item, key))
        })
        .collect::<tauri::Result<Vec<_>>>()?;
    let entries: Vec<&dyn IsMenuItem<Wry>> = items
        .iter()
        .map(|(item, _)| item as &dyn IsMenuItem<Wry>)
        .collect();
    let menu = Menu::with_items(app, &entries)?;

    let mut tray = TrayIconBuilder::with_id("main-tray")
        .menu(&menu)
        .show_menu_on_left_click(true)
        .tooltip("Hows")
        .on_menu_event(|app, event| {
            let state = app.state::<AppState>();
            match event.id.as_ref() {
                "toggle" => {
                    state.recorder.send(Command::Toggle).ok();
                }
                "pause" => {
                    state.recorder.send(Command::TogglePause).ok();
                }
                "show" => show_main_window(app.clone()),
                "quit" => {
                    app.emit("app://quit-requested", ()).ok();
                }
                _ => {}
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(TrayMenu(items))
}
