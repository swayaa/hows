//! Capture-Worker: Die Session läuft in einem dedizierten Thread, die
//! Kommunikation erfolgt über Channels (Spec §3). Unter Windows kommen die
//! echten Capture-Implementierungen zum Einsatz, auf anderen Plattformen
//! die Fakes (Entwicklungs-Modus).

use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use steps_capture::{
    now_ms, CaptureError, CapturedElement, ElementInspector, EventSink, InputListener, Point,
    RawEvent,
};
use steps_session::{
    derive_guide_title, guide_title_from, Locale, Pauses, Session, SessionState, Stamp,
};
use steps_store::{Environment, StepsFile};
use tauri::{AppHandle, Emitter};

use crate::commands::show_main_window;
use crate::error::{CommandError, ErrorCode};
use crate::settings::Settings;

#[cfg(target_os = "windows")]
type PlatformCapturer = steps_capture::windows::WindowsScreenCapturer;
#[cfg(target_os = "windows")]
type PlatformListener = steps_capture::windows::WindowsInputListener;

#[cfg(not(target_os = "windows"))]
type PlatformCapturer = steps_capture::fake::FakeScreenCapturer;
#[cfg(not(target_os = "windows"))]
type PlatformListener = steps_capture::fake::FakeInputListener;

/// Inspector-Hülle: Degradiert zu leeren Element-Daten, falls die
/// UIA-Initialisierung scheitert (statt die Aufnahme zu verhindern).
///
/// Kein `Send`-Bound: Der Inspector wird ausschließlich auf dem
/// Recorder-Thread erzeugt und verwendet (COM-Apartment-Anforderung).
struct MaybeInspector(Option<Box<dyn ElementInspector>>);

impl MaybeInspector {
    fn new() -> Self {
        #[cfg(target_os = "windows")]
        {
            match steps_capture::windows::WindowsElementInspector::new() {
                Ok(inspector) => Self(Some(Box::new(inspector))),
                Err(error) => {
                    eprintln!("UIA nicht verfügbar, Aufnahme ohne Element-Daten: {error}");
                    Self(None)
                }
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            Self(Some(Box::new(
                steps_capture::fake::FakeElementInspector::default(),
            )))
        }
    }
}

impl ElementInspector for MaybeInspector {
    fn element_at(&mut self, position: Point) -> Result<CapturedElement, CaptureError> {
        match &mut self.0 {
            Some(inspector) => inspector.element_at(position),
            None => Ok(CapturedElement::default()),
        }
    }

    fn focused_element(&mut self) -> Result<CapturedElement, CaptureError> {
        match &mut self.0 {
            Some(inspector) => inspector.focused_element(),
            None => Ok(CapturedElement::default()),
        }
    }
}

type PlatformSession = Session<PlatformCapturer, MaybeInspector>;

/// Steuer-Hotkeys der App (Toggle/Stop und Pause). Sie dürfen keine Steps
/// erzeugen.
#[derive(Debug, Clone)]
pub struct ControlHotkeys {
    /// Aufnahme starten/stoppen (Default `Ctrl+Shift+R`).
    pub toggle: String,
    /// Pause/Fortsetzen (Default `Ctrl+Shift+P`).
    pub pause: String,
}

/// Alles, was der Recorder aus den Einstellungen braucht.
#[derive(Debug, Clone)]
pub struct RecorderConfig {
    /// Steuer-Hotkeys, die keine Schritte erzeugen.
    pub hotkeys: ControlHotkeys,
    /// Sprache der Schritttexte und des Titels.
    pub locale: Locale,
    /// Pausen, nach denen Tippen und Scrollen einen neuen Schritt beginnen.
    pub pauses: Pauses,
    /// Eigene Titelvorlage; `None` nutzt die Vorlage der Sprache.
    pub title_template: Option<String>,
}

impl RecorderConfig {
    /// Liest die Recorder-Werte aus den gespeicherten Einstellungen.
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            hotkeys: ControlHotkeys {
                toggle: settings.hotkey.clone(),
                pause: settings.pause_hotkey.clone(),
            },
            locale: settings.locale(),
            pauses: settings.pauses(),
            title_template: settings.title_template().map(str::to_owned),
        }
    }

    /// Titel einer fertigen Aufnahme in der Sprache, in der sie lief;
    /// Datum und Uhrzeit in Ortszeit.
    fn title(&self, session: &PlatformSession, created_at_ms: u64) -> String {
        let stamp = Stamp::local(created_at_ms);
        match &self.title_template {
            Some(template) => guide_title_from(template, session.steps(), &stamp),
            None => derive_guide_title(session.locale(), session.steps(), &stamp),
        }
    }
}

/// `true`, wenn das Event ein konfigurierter Steuer-Hotkey ist und daher
/// nicht als Schritt erfasst werden darf.
pub(crate) fn is_control_hotkey(event: &RawEvent, hotkeys: &ControlHotkeys) -> bool {
    match event {
        RawEvent::KeyCombo { combo, .. } => {
            combo.matches_hotkey(&hotkeys.toggle) || combo.matches_hotkey(&hotkeys.pause)
        }
        RawEvent::MouseDown { .. } | RawEvent::Wheel { .. } | RawEvent::Typing { .. } => false,
    }
}

/// Kommandos an den Recorder-Thread.
pub enum Command {
    /// Aufnahme starten (aus Idle/Reviewing).
    Start,
    /// Aufnahme pausieren.
    Pause,
    /// Pause, danach Bestätigung. Schon pausiert bleibt pausiert.
    PauseSettled(Sender<()>),
    /// Aufnahme fortsetzen.
    Resume,
    /// Aufnahme beenden; Ergebnis landet im geteilten Guide-Slot.
    Stop,
    /// Aufnahme verwerfen.
    Discard,
    /// Hotkey- und Tray-Toggle. Aus Recording oder Paused nach Stop. Aus Idle oder Reviewing
    /// fragt `recorder://start-requested` das Frontend, weil nur es weiß, ob
    /// eine bearbeitete Anleitung verloren ginge.
    Toggle,
    /// Tray-Toggle. Aus Recording nach Pause, aus Paused nach Resume.
    TogglePause,
    /// Neue Konfiguration nach einer Settings-Änderung. Die Sprache greift
    /// ab der nächsten Aufnahme.
    Configure(RecorderConfig),
    /// Roh-Event aus dem Input-Hook.
    Event(RawEvent),
    /// Beenden: Hooks lösen und die Session verwerfen, ohne das Fenster zu zeigen.
    Release(Sender<()>),
}

/// Zustands-Event an das Frontend.
#[derive(Debug, Clone, Serialize)]
struct StateEvent {
    state: &'static str,
    steps: usize,
}

fn state_name(state: SessionState) -> &'static str {
    match state {
        SessionState::Idle => "idle",
        SessionState::Recording => "recording",
        SessionState::Paused => "paused",
        SessionState::Reviewing => "reviewing",
    }
}

/// Meldet `recorder://state` an das Frontend.
pub(crate) fn emit_state_named(app: &AppHandle, state: &'static str, steps: usize) {
    crate::capture_window::sync(app, state);
    app.emit("recorder://state", StateEvent { state, steps })
        .ok();
}

fn emit_state(app: &AppHandle, session: &PlatformSession) {
    emit_state_named(app, state_name(session.state()), session.steps().len());
}

/// Pro Aufnahme neu gelesen, damit geänderte Maus-Einstellungen greifen.
fn new_session(config: &RecorderConfig) -> PlatformSession {
    #[cfg(target_os = "windows")]
    let double_click = steps_capture::windows::system_double_click_threshold();
    #[cfg(not(target_os = "windows"))]
    let double_click = steps_capture::DoubleClickThreshold::default();
    Session::new(
        PlatformCapturer::default(),
        MaybeInspector::new(),
        config.locale,
    )
    .with_double_click(double_click)
    .with_pauses(config.pauses)
}

/// Startet den Recorder-Thread und liefert den Kommando-Sender. Send-Fehler
/// dürfen ignoriert werden, der Worker lebt so lange wie die App.
pub fn spawn(
    app: AppHandle,
    guide_slot: Arc<Mutex<Option<StepsFile>>>,
    config: RecorderConfig,
) -> Sender<Command> {
    let (sender, receiver) = channel::<Command>();
    let event_sender = sender.clone();

    std::thread::Builder::new()
        .name("recorder".to_owned())
        .spawn(move || {
            // Session und Inspector leben auf diesem Thread (COM-Anforderung
            // der UIA-Initialisierung unter Windows).
            let mut session = new_session(&config);
            let mut listener = PlatformListener::default();
            let mut config = config;

            for command in receiver {
                match command {
                    Command::Start => {
                        match start(&mut session, &mut listener, &event_sender, &config) {
                            Ok(()) => emit_state(&app, &session),
                            Err(error) => {
                                eprintln!("Input-Hooks nicht verfügbar: {error}");
                                show_main_window(app.clone());
                                app.emit("recorder://start-failed", error).ok();
                            }
                        }
                    }
                    Command::Pause => {
                        if session.pause().is_ok() {
                            emit_state(&app, &session);
                        }
                    }
                    Command::PauseSettled(done) => {
                        if session.pause().is_ok() {
                            emit_state(&app, &session);
                        }
                        done.send(()).ok();
                    }
                    Command::Resume => {
                        if session.resume().is_ok() {
                            emit_state(&app, &session);
                        }
                    }
                    Command::Stop => {
                        stop(&app, &mut session, &mut listener, &guide_slot, &config);
                    }
                    Command::Discard => {
                        listener.stop().ok();
                        session.discard();
                        *guide_slot.lock().expect("Guide-Lock") = None;
                        emit_state(&app, &session);
                    }
                    Command::Toggle => match session.state() {
                        SessionState::Idle | SessionState::Reviewing => {
                            app.emit("recorder://start-requested", ()).ok();
                        }
                        SessionState::Recording | SessionState::Paused => {
                            stop(&app, &mut session, &mut listener, &guide_slot, &config);
                        }
                    },
                    Command::TogglePause => {
                        let changed = match session.state() {
                            SessionState::Recording => session.pause().is_ok(),
                            SessionState::Paused => session.resume().is_ok(),
                            SessionState::Idle | SessionState::Reviewing => false,
                        };
                        if changed {
                            emit_state(&app, &session);
                        }
                    }
                    Command::Configure(next) => {
                        session.set_pauses(next.pauses);
                        config = next;
                    }
                    Command::Event(event) => {
                        // Stop-/Pause-Hotkey nicht als Step speichern.
                        if is_control_hotkey(&event, &config.hotkeys) {
                            continue;
                        }
                        if session.handle_event(event).is_some() {
                            emit_state(&app, &session);
                        }
                    }
                    Command::Release(done) => {
                        listener.stop().ok();
                        session.discard();
                        done.send(()).ok();
                    }
                }
            }
        })
        .expect("Recorder-Thread konnte nicht gestartet werden");

    sender
}

/// Startet Session und Input-Hook gemeinsam. Startet der Hook nicht, kehrt
/// die Session nach Idle zurück, damit Hows nicht als aufnehmend erscheint.
fn start(
    session: &mut PlatformSession,
    listener: &mut impl InputListener,
    event_sender: &Sender<Command>,
    config: &RecorderConfig,
) -> Result<(), CommandError> {
    // Neue Aufnahme beginnt frisch in der aktuellen Sprache; der zuletzt
    // gestoppte Guide liegt bereits im geteilten Slot.
    let fresh = match session.state() {
        SessionState::Reviewing => true,
        SessionState::Idle => session.locale() != config.locale,
        SessionState::Recording | SessionState::Paused => false,
    };
    if fresh {
        *session = new_session(config);
    }
    if session.start().is_err() {
        return Ok(());
    }
    let sink_sender = event_sender.clone();
    let sink: EventSink = Box::new(move |event| {
        // Nur weiterleiten. Das Hook-Callback darf nicht blockieren.
        sink_sender.send(Command::Event(event)).ok();
    });
    listener.start(sink).map_err(|error| {
        session.discard();
        CommandError::new(ErrorCode::InputHookFailed, error)
    })
}

fn stop(
    app: &AppHandle,
    session: &mut PlatformSession,
    listener: &mut PlatformListener,
    guide_slot: &Arc<Mutex<Option<StepsFile>>>,
    config: &RecorderConfig,
) {
    if session.stop().is_err() {
        return;
    }
    listener.stop().ok();
    emit_state(app, session);

    let finished = std::mem::replace(session, new_session(config));
    // Der frische Ersatz bleibt in Idle; das Frontend sieht "reviewing" über
    // das Event oben und lädt den Guide aus dem Slot.
    let created_at_ms = now_ms();
    let title = config.title(&finished, created_at_ms);
    match finished.into_steps_file(title, created_at_ms) {
        Ok(mut file) => {
            // Umgebungsinfo für den Bug-Report-Modus; die Windows-Version
            // liest die App noch nicht.
            file.guide.environment = Some(Environment {
                os: Some(std::env::consts::OS.to_owned()),
                os_version: None,
            });
            *guide_slot.lock().expect("Guide-Lock") = Some(file);
            app.emit("recorder://finished", ()).ok();
        }
        Err(error) => eprintln!("Guide konnte nicht erzeugt werden: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use steps_capture::fake::FakeInputListener;
    use steps_capture::KeyComboEvent;

    fn config() -> RecorderConfig {
        RecorderConfig {
            hotkeys: ControlHotkeys {
                toggle: "Ctrl+Shift+R".to_owned(),
                pause: "Ctrl+Shift+P".to_owned(),
            },
            locale: Locale::EN,
            pauses: Pauses::default(),
            title_template: None,
        }
    }

    #[test]
    fn ohne_input_hook_bleibt_die_aufnahme_idle() {
        let config = config();
        let mut session = new_session(&config);
        let mut listener = FakeInputListener::default();
        listener.fail_next = Some("SetWindowsHookExW: Zugriff verweigert".to_owned());
        let (sender, _events) = channel();

        let error = start(&mut session, &mut listener, &sender, &config).unwrap_err();
        assert_eq!(
            serde_json::to_value(&error).unwrap()["code"],
            serde_json::json!("input_hook_failed")
        );
        assert_eq!(state_name(session.state()), "idle");
        assert!(!listener.is_running());

        start(&mut session, &mut listener, &sender, &config).unwrap();
        assert_eq!(state_name(session.state()), "recording");
        assert!(listener.is_running());
    }

    fn combo(ctrl: bool, alt: bool, shift: bool, win: bool, key: &str) -> KeyComboEvent {
        KeyComboEvent {
            ctrl,
            alt,
            shift,
            win,
            key: key.to_owned(),
        }
    }

    #[test]
    fn steuer_hotkeys_werden_erkannt() {
        let hotkeys = ControlHotkeys {
            toggle: "Ctrl+Shift+R".to_owned(),
            pause: "Ctrl+Shift+P".to_owned(),
        };
        let stop = RawEvent::KeyCombo {
            combo: combo(true, false, true, false, "R"),
            timestamp_ms: 1,
        };
        let pause = RawEvent::KeyCombo {
            combo: combo(true, false, true, false, "P"),
            timestamp_ms: 2,
        };
        let other = RawEvent::KeyCombo {
            combo: combo(true, false, false, false, "S"),
            timestamp_ms: 3,
        };
        assert!(is_control_hotkey(&stop, &hotkeys));
        assert!(is_control_hotkey(&pause, &hotkeys));
        assert!(!is_control_hotkey(&other, &hotkeys));
        assert!(!is_control_hotkey(
            &RawEvent::Typing { timestamp_ms: 4 },
            &hotkeys
        ));
    }
}
