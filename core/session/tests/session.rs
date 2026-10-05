//! Tests der Aufnahme-Statemachine und des Step-Assembly gegen die
//! Fake-Implementierungen der Capture-Traits.

use std::cell::Cell;
use std::rc::Rc;

use steps_capture::fake::{FakeElementInspector, FakeScreenCapturer, FAKE_PNG};
use steps_capture::{
    CaptureError, CapturedElement, DoubleClickThreshold, ElementInspector, KeyComboEvent,
    MouseButton, Point, RawEvent,
};
use steps_session::{
    derive_guide_title, Locale, Pauses, Session, SessionError, SessionState, Stamp, StepChange,
    SCROLL_GAP_MS, TYPING_GAP_MS,
};

const DOUBLE_CLICK_MS: u64 = 500;
const NOTCH: i32 = 120;
use steps_store::{Action, ScrollDirection, StepsFile};

type TestSession = Session<FakeScreenCapturer, FakeElementInspector>;

fn button_element() -> CapturedElement {
    CapturedElement {
        name: Some("Speichern".to_owned()),
        control_type: Some("Button".to_owned()),
        automation_id: Some("btnSave".to_owned()),
        window_title: Some("Einstellungen".to_owned()),
        app_name: Some("Beispiel-App".to_owned()),
    }
}

fn session_de() -> TestSession {
    Session::new(
        FakeScreenCapturer::default(),
        FakeElementInspector::returning(button_element()),
        Locale::resolve("de"),
    )
}

fn recording_de() -> TestSession {
    let mut session = session_de();
    session.start().unwrap();
    session
}

fn click_at(x: i32, y: i32, timestamp_ms: u64) -> RawEvent {
    RawEvent::MouseDown {
        position: Point { x, y },
        button: MouseButton::Left,
        timestamp_ms,
    }
}

const WINDOW_PID: u32 = 1;

fn wheel(delta: i32, horizontal: bool, timestamp_ms: u64) -> RawEvent {
    wheel_at(
        delta,
        horizontal,
        timestamp_ms,
        Some("Einstellungen".to_owned()),
        WINDOW_PID,
    )
}

fn wheel_at(
    delta: i32,
    horizontal: bool,
    timestamp_ms: u64,
    window_title: Option<String>,
    window_pid: u32,
) -> RawEvent {
    RawEvent::Wheel {
        position: Point { x: 100, y: 100 },
        delta,
        horizontal,
        timestamp_ms,
        window_title,
        window_pid,
    }
}

/// Zählt `element_at`, damit ein Folgetick ohne UI Automation sichtbar wird.
struct CountingInspector {
    inner: FakeElementInspector,
    hits: Rc<Cell<usize>>,
}

impl ElementInspector for CountingInspector {
    fn element_at(&mut self, position: Point) -> Result<CapturedElement, CaptureError> {
        self.hits.set(self.hits.get() + 1);
        self.inner.element_at(position)
    }

    fn focused_element(&mut self) -> Result<CapturedElement, CaptureError> {
        self.inner.focused_element()
    }
}

// --- Statemachine -----------------------------------------------------------

#[test]
fn statemachine_erlaubt_nur_gueltige_uebergaenge() {
    let mut session = session_de();
    assert_eq!(session.state(), SessionState::Idle);

    // Ungültig aus Idle:
    assert!(matches!(
        session.pause(),
        Err(SessionError::InvalidTransition { .. })
    ));
    assert!(matches!(
        session.resume(),
        Err(SessionError::InvalidTransition { .. })
    ));
    assert!(matches!(
        session.stop(),
        Err(SessionError::InvalidTransition { .. })
    ));

    session.start().unwrap();
    assert_eq!(session.state(), SessionState::Recording);
    assert!(matches!(
        session.start(),
        Err(SessionError::InvalidTransition { .. })
    ));

    session.pause().unwrap();
    assert_eq!(session.state(), SessionState::Paused);
    session.resume().unwrap();
    assert_eq!(session.state(), SessionState::Recording);

    session.stop().unwrap();
    assert_eq!(session.state(), SessionState::Reviewing);
    assert!(matches!(
        session.stop(),
        Err(SessionError::InvalidTransition { .. })
    ));

    session.discard();
    assert_eq!(session.state(), SessionState::Idle);
    assert_eq!(session.steps(), []);
}

#[test]
fn stop_aus_paused_ist_erlaubt() {
    let mut session = recording_de();
    session.pause().unwrap();
    session.stop().unwrap();
    assert_eq!(session.state(), SessionState::Reviewing);
}

#[test]
fn events_ausserhalb_von_recording_werden_ignoriert() {
    let mut session = session_de();
    assert_eq!(session.handle_event(click_at(10, 10, 1000)), None);

    session.start().unwrap();
    session.pause().unwrap();
    assert_eq!(session.handle_event(click_at(10, 10, 2000)), None);

    session.resume().unwrap();
    assert!(session.handle_event(click_at(10, 10, 3000)).is_some());
    assert_eq!(session.steps().len(), 1);
}

// --- Klicks -----------------------------------------------------------------

#[test]
fn klick_erzeugt_schritt_mit_screenshot_element_und_text() {
    let mut session = recording_de();
    let change = session.handle_event(click_at(300, 200, 1000));
    assert_eq!(change, Some(StepChange::Created(0)));

    let step = &session.steps()[0];
    assert_eq!(step.action, Action::Click);
    assert_eq!(step.timestamp_ms, 1000);
    assert_eq!(step.position.unwrap().x, 300);
    let monitor = step.monitor.as_ref().unwrap();
    assert_eq!(monitor.width, 1920);
    assert!((monitor.scale_factor - 1.0).abs() < f64::EPSILON);
    let element = step.element.as_ref().unwrap();
    assert_eq!(element.name.as_deref(), Some("Speichern"));
    assert_eq!(
        step.text.generated,
        "Klicke auf „Speichern“ in „Einstellungen“"
    );
    assert_eq!(session.images().get(&step.id).unwrap(), FAKE_PNG);
}

#[test]
fn rechtsklick_erzeugt_eigenen_aktionstyp() {
    let mut session = recording_de();
    session.handle_event(RawEvent::MouseDown {
        position: Point { x: 5, y: 5 },
        button: MouseButton::Right,
        timestamp_ms: 1000,
    });
    let step = &session.steps()[0];
    assert_eq!(step.action, Action::RightClick);
    assert_eq!(
        step.text.generated,
        "Klicke mit der rechten Maustaste auf „Speichern“ in „Einstellungen“"
    );
}

#[test]
fn mittlere_maustaste_wird_ignoriert() {
    let mut session = recording_de();
    let change = session.handle_event(RawEvent::MouseDown {
        position: Point { x: 5, y: 5 },
        button: MouseButton::Middle,
        timestamp_ms: 1000,
    });
    assert_eq!(change, None);
    assert_eq!(session.steps(), []);
}

#[test]
fn klick_ohne_element_laesst_text_leer_screenshot_bleibt() {
    let mut session = Session::new(
        FakeScreenCapturer::default(),
        FakeElementInspector::default(),
        Locale::resolve("de"),
    );
    session.start().unwrap();
    session.handle_event(click_at(42, 7, 1000));
    let step = &session.steps()[0];
    assert_eq!(step.text.generated, "");
    assert!(session.images().contains_key(&step.id));
}

#[test]
fn screenshot_fehler_verhindert_den_schritt_nicht() {
    let capturer = FakeScreenCapturer {
        fail_next: Some("kein Monitor".to_owned()),
        ..FakeScreenCapturer::default()
    };
    let mut session = Session::new(
        capturer,
        FakeElementInspector::returning(button_element()),
        Locale::resolve("de"),
    );
    session.start().unwrap();
    session.handle_event(click_at(1, 1, 1000));

    let step = &session.steps()[0];
    assert_eq!(step.action, Action::Click);
    assert!(step.monitor.is_none());
    assert!(session.images().is_empty());
}

// --- Doppelklick ------------------------------------------------------------

#[test]
fn zwei_nahe_klicks_werden_zum_doppelklick_zusammengefuehrt() {
    let mut session = recording_de();
    session.handle_event(click_at(100, 100, 1000));
    let change = session.handle_event(click_at(103, 98, 1000 + DOUBLE_CLICK_MS));

    assert_eq!(change, Some(StepChange::Updated(0)));
    assert_eq!(session.steps().len(), 1);
    let step = &session.steps()[0];
    assert_eq!(step.action, Action::DoubleClick);
    assert_eq!(
        step.text.generated,
        "Doppelklicke auf „Speichern“ in „Einstellungen“"
    );
}

#[test]
fn doppelklick_behaelt_den_ersten_screenshot() {
    let mut session = recording_de();
    session.handle_event(click_at(100, 100, 1000));
    session.handle_event(click_at(100, 100, 1200));

    assert_eq!(session.steps().len(), 1);
    assert_eq!(session.images().len(), 1);
}

#[test]
fn zeitlich_entfernte_klicks_bleiben_getrennt() {
    let mut session = recording_de();
    session.handle_event(click_at(100, 100, 1000));
    session.handle_event(click_at(100, 100, 1000 + DOUBLE_CLICK_MS + 1));
    assert_eq!(session.steps().len(), 2);
    assert_eq!(session.steps()[0].action, Action::Click);
    assert_eq!(session.steps()[1].action, Action::Click);
}

#[test]
fn raeumlich_entfernte_klicks_bleiben_getrennt() {
    let mut session = recording_de();
    session.handle_event(click_at(100, 100, 1000));
    session.handle_event(click_at(200, 100, 1100));
    assert_eq!(session.steps().len(), 2);
}

#[test]
fn doppelklick_nutzt_die_gesetzten_systemschwellen() {
    // System: 300 ms, 4×4 px (±2 px um den ersten Klick).
    let mut session = session_de().with_double_click(DoubleClickThreshold {
        interval_ms: 300,
        width_px: 4,
        height_px: 4,
    });
    session.start().unwrap();
    session.handle_event(click_at(100, 100, 1000));
    session.handle_event(click_at(103, 100, 1100)); // 3 px daneben
    session.handle_event(click_at(103, 100, 1500)); // 400 ms später
    assert_eq!(session.steps().len(), 3);

    session.handle_event(click_at(104, 101, 1700)); // ±1 px, 200 ms
    assert_eq!(session.steps().len(), 3);
    assert_eq!(session.steps()[2].action, Action::DoubleClick);
}

#[test]
fn dreifachklick_ergibt_doppelklick_plus_klick() {
    let mut session = recording_de();
    session.handle_event(click_at(100, 100, 1000));
    session.handle_event(click_at(100, 100, 1100));
    session.handle_event(click_at(100, 100, 1200));
    assert_eq!(session.steps().len(), 2);
    assert_eq!(session.steps()[0].action, Action::DoubleClick);
    assert_eq!(session.steps()[1].action, Action::Click);
}

// --- Scroll-Drosselung ------------------------------------------------------

#[test]
fn scroll_ticks_gleicher_richtung_werden_zu_einem_schritt_gedrosselt() {
    let mut session = recording_de();
    session.handle_event(wheel(-NOTCH, false, 1000));
    session.handle_event(wheel(-2 * NOTCH, false, 1050));
    let change = session.handle_event(wheel(-NOTCH, false, 1100));

    assert_eq!(change, Some(StepChange::Updated(0)));
    assert_eq!(session.steps().len(), 1);
    assert_eq!(
        session.steps()[0].action,
        Action::Scroll {
            direction: ScrollDirection::Down,
            amount: 4,
        }
    );
    assert_eq!(
        session.steps()[0].text.generated,
        "Scrolle 4 in „Einstellungen“ nach unten"
    );
    // Nur ein Screenshot für die ganze Gruppe.
    assert_eq!(session.images().len(), 1);
}

#[test]
fn richtungswechsel_beginnt_neuen_scroll_schritt() {
    let mut session = recording_de();
    session.handle_event(wheel(-3 * NOTCH, false, 1000));
    session.handle_event(wheel(2 * NOTCH, false, 1100));

    assert_eq!(session.steps().len(), 2);
    assert_eq!(
        session.steps()[1].action,
        Action::Scroll {
            direction: ScrollDirection::Up,
            amount: 2,
        }
    );
}

#[test]
fn elementwechsel_im_selben_fenster_bleibt_ein_scroll_schritt() {
    // Beim Scrollen läuft Inhalt unter dem Zeiger durch: anderes Element,
    // gleiches Fenster, gleicher Prozess. Der Satz nennt das Fenster.
    let mut inspector = FakeElementInspector::returning(button_element());
    inspector.queue.push_back(CapturedElement {
        name: Some("Liste".to_owned()),
        ..button_element()
    });
    let hits = Rc::new(Cell::new(0));
    let mut session = Session::new(
        FakeScreenCapturer::default(),
        CountingInspector {
            inner: inspector,
            hits: Rc::clone(&hits),
        },
        Locale::resolve("de"),
    );
    session.start().unwrap();

    session.handle_event(wheel(-NOTCH, false, 1000)); // Element „Liste“
    session.handle_event(wheel(-NOTCH, false, 1050)); // wäre „Speichern“, wird nicht abgefragt

    assert_eq!(session.steps().len(), 1);
    assert_eq!(hits.get(), 1);
    assert_eq!(session.images().len(), 1);
    assert_eq!(
        session.steps()[0].text.generated,
        "Scrolle 2 in „Einstellungen“ nach unten"
    );
    assert!(!session.steps()[0].text.generated.contains("Liste"));
    assert!(!session.steps()[0].text.generated.contains("Speichern"));
}

#[test]
fn fensterwechsel_beginnt_neuen_scroll_schritt() {
    let mut session = recording_de();
    session.handle_event(wheel(-NOTCH, false, 1000));
    session.handle_event(wheel_at(
        -NOTCH,
        false,
        1050,
        Some("Anderes Fenster".to_owned()),
        WINDOW_PID,
    ));

    assert_eq!(session.steps().len(), 2);
    assert_eq!(session.images().len(), 2);
}

#[test]
fn anderer_prozess_beginnt_neuen_scroll_schritt() {
    let mut session = recording_de();
    session.handle_event(wheel(-NOTCH, false, 1000));
    session.handle_event(wheel_at(
        -NOTCH,
        false,
        1050,
        Some("Einstellungen".to_owned()),
        WINDOW_PID + 1,
    ));

    assert_eq!(session.steps().len(), 2);
}

#[test]
fn scroll_pause_beginnt_neuen_schritt() {
    let mut session = recording_de();
    session.handle_event(wheel(-NOTCH, false, 1000));
    session.handle_event(wheel(-NOTCH, false, 1000 + SCROLL_GAP_MS));
    assert_eq!(session.steps().len(), 1);

    session.handle_event(wheel(-NOTCH, false, 1001 + 2 * SCROLL_GAP_MS));
    assert_eq!(session.steps().len(), 2);
}

#[test]
fn touchpad_bruchteile_werden_zu_rastern_summiert() {
    let mut session = recording_de();
    // Zehn Touchpad-Events à 30 Einheiten sind 300, aufgerundet 3 Raster.
    for i in 0..10 {
        session.handle_event(wheel(-30, false, 1000 + i * 20));
    }
    assert_eq!(session.steps().len(), 1);
    assert_eq!(
        session.steps()[0].action,
        Action::Scroll {
            direction: ScrollDirection::Down,
            amount: 3,
        }
    );
}

#[test]
fn horizontales_scrollen_wird_erfasst() {
    let mut session = recording_de();
    session.handle_event(wheel(NOTCH, true, 1000));
    session.handle_event(wheel(-2 * NOTCH, true, 1100));

    assert_eq!(
        session.steps()[0].action,
        Action::Scroll {
            direction: ScrollDirection::Right,
            amount: 1,
        }
    );
    assert_eq!(
        session.steps()[1].action,
        Action::Scroll {
            direction: ScrollDirection::Left,
            amount: 2,
        }
    );
}

#[test]
fn klick_beendet_scroll_gruppe() {
    let mut session = recording_de();
    session.handle_event(wheel(-NOTCH, false, 1000));
    session.handle_event(click_at(10, 10, 1100));
    session.handle_event(wheel(-NOTCH, false, 1200));

    assert_eq!(session.steps().len(), 3);
}

// --- Key-Combos (nie Klartext, Spec §5) --------------------------------------

// Spiegelt bewusst die vier Modifier-Flags von `KeyComboEvent` wider.
#[allow(clippy::fn_params_excessive_bools)]
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
fn key_combo_mit_strg_wird_als_schritt_erfasst() {
    let mut session = recording_de();
    session.handle_event(RawEvent::KeyCombo {
        combo: combo(true, false, false, false, "S"),
        timestamp_ms: 1000,
    });

    let step = &session.steps()[0];
    assert_eq!(
        step.action,
        Action::KeyCombo {
            combo: "Ctrl+S".to_owned()
        }
    );
    assert_eq!(step.text.generated, "Drücke Strg+S");
}

#[test]
fn key_combo_reihenfolge_ist_kanonisch() {
    let mut session = recording_de();
    session.handle_event(RawEvent::KeyCombo {
        combo: combo(true, true, true, true, "Esc"),
        timestamp_ms: 1000,
    });
    assert_eq!(
        session.steps()[0].action,
        Action::KeyCombo {
            combo: "Ctrl+Alt+Shift+Win+Esc".to_owned()
        }
    );
}

#[test]
fn taste_ohne_echten_modifier_erzeugt_nie_einen_klartext_schritt() {
    let mut session = recording_de();
    // Shift allein ist kein echter Modifier: „Shift+A“ wäre Klartext.
    session.handle_event(RawEvent::KeyCombo {
        combo: combo(false, false, true, false, "A"),
        timestamp_ms: 1000,
    });
    session.handle_event(RawEvent::KeyCombo {
        combo: combo(false, false, false, false, "B"),
        timestamp_ms: 1100,
    });

    assert_eq!(session.steps().len(), 1);
    let step = &session.steps()[0];
    assert_eq!(step.action, Action::TextInput);
    // Weder Tastenname noch Inhalt tauchen irgendwo auf.
    let json = serde_json::to_string(&session.steps()[0]).unwrap();
    assert!(!json.contains("\"A\"") && !json.contains("\"B\""));
    assert_eq!(step.text.generated, "Gib deinen Text in „Speichern“ ein");
}

// --- Texteingabe-Zusammenfassung ---------------------------------------------

#[test]
fn tippen_wird_zu_einem_schritt_zusammengefasst() {
    let mut session = recording_de();
    session.handle_event(RawEvent::Typing { timestamp_ms: 1000 });
    session.handle_event(RawEvent::Typing { timestamp_ms: 1500 });
    let change = session.handle_event(RawEvent::Typing { timestamp_ms: 2000 });

    assert_eq!(change, Some(StepChange::Updated(0)));
    assert_eq!(session.steps().len(), 1);
    assert_eq!(session.steps()[0].action, Action::TextInput);
}

#[test]
fn tipp_pause_beginnt_neuen_texteingabe_schritt() {
    let mut session = recording_de();
    session.handle_event(RawEvent::Typing { timestamp_ms: 1000 });
    session.handle_event(RawEvent::Typing {
        timestamp_ms: 1000 + TYPING_GAP_MS + 1,
    });
    assert_eq!(session.steps().len(), 2);
}

#[test]
fn eingestellte_pausen_gelten_auch_waehrend_der_aufnahme() {
    let mut session = recording_de().with_pauses(Pauses {
        typing_ms: 500,
        scroll_ms: 200,
    });
    session.handle_event(RawEvent::Typing { timestamp_ms: 1000 });
    session.handle_event(RawEvent::Typing { timestamp_ms: 1501 });
    assert_eq!(session.steps().len(), 2, "typing pause of 500 ms");

    session.set_pauses(Pauses::default());
    session.handle_event(wheel(-NOTCH, false, 3000));
    session.handle_event(wheel(-NOTCH, false, 3000 + SCROLL_GAP_MS));
    assert_eq!(session.steps().len(), 3, "back to the default scroll pause");
}

#[test]
fn klick_beendet_texteingabe_gruppe() {
    let mut session = recording_de();
    session.handle_event(RawEvent::Typing { timestamp_ms: 1000 });
    session.handle_event(click_at(10, 10, 1100));
    session.handle_event(RawEvent::Typing { timestamp_ms: 1200 });
    assert_eq!(session.steps().len(), 3);
}

// --- Manuelle Notiz ----------------------------------------------------------

#[test]
fn manuelle_notiz_in_recording_und_reviewing() {
    let mut session = recording_de();
    session.add_manual_note("Vorbereitung: VPN verbinden", 1000);
    session.stop().unwrap();
    session.add_manual_note("Nachbereitung", 2000);

    assert_eq!(session.steps().len(), 2);
    assert_eq!(session.steps()[0].action, Action::ManualNote);
    assert_eq!(
        session.steps()[0].text.generated,
        "Vorbereitung: VPN verbinden"
    );
    assert!(session.steps()[0].position.is_none());
}

#[test]
fn manuelle_notiz_in_idle_wird_abgelehnt() {
    let mut session = session_de();
    assert_eq!(session.add_manual_note("nix", 0), None);
}

// --- Abschluss & Integration mit dem Store ------------------------------------

#[test]
fn into_steps_file_nur_im_reviewing_zustand() {
    let session = recording_de();
    assert!(matches!(
        session.into_steps_file("Titel", 0),
        Err(SessionError::NotReviewing { .. })
    ));
}

#[test]
fn aufnahme_laesst_sich_als_steps_datei_speichern_und_wieder_laden() {
    let mut session = recording_de();
    session.handle_event(click_at(100, 100, 1000));
    session.handle_event(RawEvent::KeyCombo {
        combo: combo(true, false, false, false, "S"),
        timestamp_ms: 2000,
    });
    session.stop().unwrap();

    let file = session
        .into_steps_file("Meine Anleitung", 3000)
        .expect("StepsFile");
    assert_eq!(file.guide.language, "de");
    assert_eq!(file.guide.title, "Meine Anleitung");
    assert_eq!(file.guide.steps.len(), 2);

    let mut buffer = std::io::Cursor::new(Vec::new());
    file.write_to(&mut buffer).expect("Schreiben");
    buffer.set_position(0);
    let read_back = StepsFile::read_from(buffer).expect("Lesen");
    assert_eq!(read_back, file);
}

#[test]
fn generic_uia_container_ohne_name_nutzt_fenster_nicht_control_type() {
    let mut session = Session::new(
        FakeScreenCapturer::default(),
        FakeElementInspector::returning(CapturedElement {
            name: None,
            control_type: Some("Group".to_owned()),
            window_title: Some("Microsoft Edge".to_owned()),
            ..CapturedElement::default()
        }),
        Locale::resolve("en"),
    );
    session.start().unwrap();
    session.handle_event(click_at(10, 10, 1000));

    let generated = &session.steps()[0].text.generated;
    assert_eq!(generated, "Click in “Microsoft Edge”");
    assert!(!generated.contains("Group"));

    session.stop().unwrap();
    let title = derive_guide_title(
        Locale::resolve("en"),
        session.steps(),
        &Stamp::with_offset(2000, 0),
    );
    let file = session.into_steps_file(title, 2000).expect("StepsFile");
    assert_eq!(file.guide.title, "");
}

// --- Englische Templates -------------------------------------------------------

#[test]
fn englische_templates_sind_verfuegbar() {
    let mut session = Session::new(
        FakeScreenCapturer::default(),
        FakeElementInspector::returning(button_element()),
        Locale::resolve("en"),
    );
    session.start().unwrap();
    session.handle_event(click_at(1, 1, 1000));
    session.handle_event(RawEvent::KeyCombo {
        combo: combo(true, false, true, false, "S"),
        timestamp_ms: 2000,
    });
    session.handle_event(wheel(-NOTCH, false, 3000));
    session.handle_event(RawEvent::Typing { timestamp_ms: 4000 });

    assert_eq!(
        session.steps()[0].text.generated,
        "Click “Speichern” in “Einstellungen”"
    );
    assert_eq!(session.steps()[1].text.generated, "Press Ctrl+Shift+S");
    assert_eq!(
        session.steps()[2].text.generated,
        "Scroll down 1 in “Einstellungen”"
    );
    assert_eq!(
        session.steps()[3].text.generated,
        "Type your text in “Speichern”"
    );
}
