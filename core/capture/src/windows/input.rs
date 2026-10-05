//! Globale Low-Level-Input-Hooks via `SetWindowsHookEx` (Spec §5).
//!
//! Die Hooks laufen auf einem eigenen Thread mit Message-Loop. Events gehen
//! an den registrierten [`EventSink`]. Der Sink sollte nur in einen Channel
//! weiterleiten. Das Hook-Callback blockiert sonst die globale Eingabe.
//!
//! Tastendrücke ohne echten Modifier (Strg, Alt oder Win) werden hier zu
//! [`RawEvent::Typing`] ohne Tastenname. Der Tastenname verlässt diese
//! Funktion nicht. Das gilt auch für Alt Gr, die rechte Alt-Taste. Windows
//! meldet sie als Strg+Alt, sie tippt aber Zeichen wie `@` oder `€`.
//! Tastendrücke in einem Fenster von Hows werden nicht gemeldet.

use std::sync::mpsc;
use std::sync::Mutex;
use std::thread::JoinHandle;

use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::Threading::{GetCurrentProcessId, GetCurrentThreadId};
use windows::Win32::UI::Input::KeyboardAndMouse::{GetAsyncKeyState, GetDoubleClickTime};
use windows::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetAncestor, GetForegroundWindow, GetMessageW,
    GetSystemMetrics, GetWindowThreadProcessId, PostThreadMessageW, SetWindowsHookExW,
    TranslateMessage, UnhookWindowsHookEx, WindowFromPoint, GA_ROOT, KBDLLHOOKSTRUCT, MSG,
    MSLLHOOKSTRUCT, SM_CXDOUBLECLK, SM_CYDOUBLECLK, WH_KEYBOARD_LL, WH_MOUSE_LL, WM_KEYDOWN,
    WM_LBUTTONDOWN, WM_MBUTTONDOWN, WM_MOUSEHWHEEL, WM_MOUSEWHEEL, WM_QUIT, WM_RBUTTONDOWN,
    WM_SYSKEYDOWN,
};

use super::now_ms;
use super::uia::window_title;
use crate::{
    CaptureError, DoubleClickThreshold, EventSink, InputListener, KeyComboEvent, MouseButton,
    Point, RawEvent,
};

/// Globaler Sink: Low-Level-Hook-Callbacks erhalten keinen Kontext-Zeiger,
/// daher ist genau ein aktiver Listener pro Prozess möglich.
static SINK: Mutex<Option<EventSink>> = Mutex::new(None);

fn emit(event: RawEvent) {
    if let Ok(mut guard) = SINK.lock() {
        if let Some(sink) = guard.as_mut() {
            sink(event);
        }
    }
}

/// `true`, wenn das Fenster zum eigenen Prozess gehört (Recorder-Self-UI).
///
/// Klicks und Scroll auf die eigene Tauri-Oberfläche sollen keine
/// Schritte erzeugen. Erkannt über das Fensterhandle und die Prozess-ID.
pub(crate) fn is_own_process_window(hwnd: HWND) -> bool {
    if hwnd.is_invalid() {
        return false;
    }
    // SAFETY: GetWindowThreadProcessId akzeptiert beliebige HWNDs; bei
    // ungültigem Handle bleibt `window_pid` 0.
    unsafe {
        let mut window_pid = 0u32;
        let _ = GetWindowThreadProcessId(hwnd, Some(&raw mut window_pid));
        window_pid != 0 && window_pid == GetCurrentProcessId()
    }
}

/// Oberstes Fenster unter dem Punkt. Der Treffer ist oft das WebView2-Kindfenster.
fn root_window_at(point: windows::Win32::Foundation::POINT) -> HWND {
    // SAFETY: WindowFromPoint und GetAncestor sind reine Abfragen.
    let hit = unsafe { WindowFromPoint(point) };
    let root = unsafe { GetAncestor(hit, GA_ROOT) };
    if root.is_invalid() {
        hit
    } else {
        root
    }
}

/// Prozess-ID des Fensters. `0`, wenn das Handle nichts liefert.
fn window_pid(hwnd: HWND) -> u32 {
    if hwnd.is_invalid() {
        return 0;
    }
    // SAFETY: GetWindowThreadProcessId akzeptiert beliebige HWNDs.
    unsafe {
        let mut pid = 0u32;
        let _ = GetWindowThreadProcessId(hwnd, Some(&raw mut pid));
        pid
    }
}

/// Extrahiert das Roh-Delta aus dem `mouseData`-Feld (HIWORD, vorzeichen-
/// behaftet). Nicht auf ganze `WHEEL_DELTA` (120) runden: Touchpads und
/// hochauflösende Räder liefern kleinere Werte, die sonst zu 0 würden.
fn wheel_delta(mouse_data: u32) -> i32 {
    // Bit-Reinterpretation des HIWORD als vorzeichenbehafteter Wert.
    let hiword = ((mouse_data >> 16) & 0xFFFF) as u16;
    i32::from(i16::from_ne_bytes(hiword.to_ne_bytes()))
}

/// Doppelklick-Schwellen aus den Systemeinstellungen (Maus-Systemsteuerung).
/// `GetSystemMetrics` liefert die Werte für die System-DPI, passend zu den
/// physischen Hook-Koordinaten des DPI-aware Prozesses.
pub fn system_double_click_threshold() -> DoubleClickThreshold {
    // SAFETY: Beide Funktionen lesen nur Systemeinstellungen.
    let (interval_ms, width_px, height_px) = unsafe {
        (
            GetDoubleClickTime(),
            GetSystemMetrics(SM_CXDOUBLECLK),
            GetSystemMetrics(SM_CYDOUBLECLK),
        )
    };
    let fallback = DoubleClickThreshold::default();
    DoubleClickThreshold {
        interval_ms: if interval_ms == 0 {
            fallback.interval_ms
        } else {
            u64::from(interval_ms)
        },
        width_px: if width_px > 0 {
            width_px
        } else {
            fallback.width_px
        },
        height_px: if height_px > 0 {
            height_px
        } else {
            fallback.height_px
        },
    }
}

/// Virtual-Key-Codes der reinen Modifier-Tasten (werden als eigenständige
/// Tastendrücke ignoriert; ihr Zustand fließt über [`modifier_state`] ein).
fn is_modifier_vk(vk: u32) -> bool {
    matches!(
        vk,
        0x10..=0x12 // VK_SHIFT, VK_CONTROL, VK_MENU
        | 0x5B | 0x5C // VK_LWIN, VK_RWIN
        | 0xA0..=0xA5 // VK_LSHIFT..VK_RMENU
    )
}

/// Kanonischer, sprachneutraler Name einer Taste (nie Text-Inhalt; wird nur
/// für Kombinationen mit echtem Modifier verwendet).
fn vk_to_key_name(vk: u32) -> String {
    match vk {
        // Ziffern 0-9 und Buchstaben A-Z entsprechen ihrem ASCII-Zeichen.
        0x30..=0x39 | 0x41..=0x5A => {
            char::from_u32(vk).map_or_else(|| format!("VK{vk:#04X}"), |c| c.to_string())
        }
        // F1-F24
        0x70..=0x87 => format!("F{}", vk - 0x6F),
        0x08 => "Backspace".to_owned(),
        0x09 => "Tab".to_owned(),
        0x0D => "Enter".to_owned(),
        0x13 => "Pause".to_owned(),
        0x14 => "CapsLock".to_owned(),
        0x1B => "Esc".to_owned(),
        0x20 => "Space".to_owned(),
        0x21 => "PageUp".to_owned(),
        0x22 => "PageDown".to_owned(),
        0x23 => "End".to_owned(),
        0x24 => "Home".to_owned(),
        0x25 => "Left".to_owned(),
        0x26 => "Up".to_owned(),
        0x27 => "Right".to_owned(),
        0x28 => "Down".to_owned(),
        0x2C => "PrintScreen".to_owned(),
        0x2D => "Insert".to_owned(),
        0x2E => "Delete".to_owned(),
        0x60..=0x69 => format!("Numpad{}", vk - 0x60),
        0x6A => "NumpadMultiply".to_owned(),
        0x6B => "NumpadPlus".to_owned(),
        0x6D => "NumpadMinus".to_owned(),
        0x6E => "NumpadDecimal".to_owned(),
        0x6F => "NumpadDivide".to_owned(),
        0x90 => "NumLock".to_owned(),
        0x91 => "ScrollLock".to_owned(),
        0xBB => "Plus".to_owned(),
        0xBC => "Comma".to_owned(),
        0xBD => "Minus".to_owned(),
        0xBE => "Period".to_owned(),
        // OEM-/Layout-abhängige Tasten neutral benennen.
        other => format!("VK{other:#04X}"),
    }
}

fn key_down(vk: i32) -> bool {
    // SAFETY: GetAsyncKeyState ist für beliebige VK-Werte definiert.
    // Höchstes Bit gesetzt (= negativ) bedeutet: Taste ist gedrückt.
    unsafe { GetAsyncKeyState(vk) < 0 }
}

/// Zustand der Modifier-Tasten beim Tastendruck.
// Vier unabhängige Tasten plus AltGr; eine Enum-Modellierung wäre künstlich.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Default)]
struct Modifiers {
    ctrl: bool,
    alt: bool,
    shift: bool,
    win: bool,
    /// Rechte Alt-Taste; Windows setzt dabei zusätzlich Strg und Alt.
    altgr: bool,
}

/// Aktueller Zustand der Modifier-Tasten.
fn modifier_state() -> Modifiers {
    Modifiers {
        ctrl: key_down(0x11),                  // VK_CONTROL
        alt: key_down(0x12),                   // VK_MENU
        shift: key_down(0x10),                 // VK_SHIFT
        win: key_down(0x5B) || key_down(0x5C), // VK_LWIN / VK_RWIN
        altgr: key_down(0xA5),                 // VK_RMENU
    }
}

/// Ordnet einen Tastendruck ein. Reine Modifier ergeben kein Event; mit Alt Gr
/// oder ohne Strg/Alt/Win wird nur [`RawEvent::Typing`] ohne Tastenname
/// gemeldet.
fn keyboard_event(vk: u32, modifiers: Modifiers, timestamp_ms: u64) -> Option<RawEvent> {
    if is_modifier_vk(vk) {
        return None;
    }
    let Modifiers {
        ctrl,
        alt,
        shift,
        win,
        altgr,
    } = modifiers;
    if win || (!altgr && (ctrl || alt)) {
        Some(RawEvent::KeyCombo {
            combo: KeyComboEvent {
                ctrl,
                alt,
                shift,
                win,
                key: vk_to_key_name(vk),
            },
            timestamp_ms,
        })
    } else {
        Some(RawEvent::Typing { timestamp_ms })
    }
}

// SAFETY: `lparam` zeigt für WH_MOUSE_LL auf eine gültige MSLLHOOKSTRUCT
// (Garantie der Win32-Hook-API), solange `code >= 0`.
unsafe extern "system" fn mouse_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let info = &*(lparam.0 as *const MSLLHOOKSTRUCT);
        // Self-UI (eigenes HWND/Prozess) nicht aufnehmen.
        let root = root_window_at(info.pt);
        if !is_own_process_window(root) {
            let position = Point {
                x: info.pt.x,
                y: info.pt.y,
            };
            let timestamp_ms = now_ms();
            #[allow(clippy::cast_possible_truncation)]
            let message = wparam.0 as u32;
            let event = match message {
                WM_LBUTTONDOWN => Some(RawEvent::MouseDown {
                    position,
                    button: MouseButton::Left,
                    timestamp_ms,
                }),
                WM_RBUTTONDOWN => Some(RawEvent::MouseDown {
                    position,
                    button: MouseButton::Right,
                    timestamp_ms,
                }),
                WM_MBUTTONDOWN => Some(RawEvent::MouseDown {
                    position,
                    button: MouseButton::Middle,
                    timestamp_ms,
                }),
                WM_MOUSEWHEEL | WM_MOUSEHWHEEL => {
                    let delta = wheel_delta(info.mouseData);
                    (delta != 0).then_some(RawEvent::Wheel {
                        position,
                        delta,
                        horizontal: message == WM_MOUSEHWHEEL,
                        timestamp_ms,
                        window_title: window_title(root),
                        window_pid: window_pid(root),
                    })
                }
                _ => None,
            };
            if let Some(event) = event {
                emit(event);
            }
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

// SAFETY: `lparam` zeigt für WH_KEYBOARD_LL auf eine gültige KBDLLHOOKSTRUCT
// (Garantie der Win32-Hook-API), solange `code >= 0`.
unsafe extern "system" fn keyboard_hook(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        #[allow(clippy::cast_possible_truncation)]
        let message = wparam.0 as u32;
        // Self-UI: Eingaben in Hows selbst sind keine Schritte.
        if (message == WM_KEYDOWN || message == WM_SYSKEYDOWN)
            && !is_own_process_window(GetForegroundWindow())
        {
            let info = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
            if let Some(event) = keyboard_event(info.vkCode, modifier_state(), now_ms()) {
                emit(event);
            }
        }
    }
    CallNextHookEx(None, code, wparam, lparam)
}

/// [`InputListener`] für Windows: `WH_MOUSE_LL`- und `WH_KEYBOARD_LL`-Hooks
/// auf einem dedizierten Message-Loop-Thread.
///
/// Hinweis DPI: Die Hook-Koordinaten sind nur dann durchgängig physische
/// Pixel, wenn der Prozess Per-Monitor-DPI-v2-aware ist (App-Manifest der
/// Tauri-App, Spec §5).
#[derive(Default)]
pub struct WindowsInputListener {
    thread: Option<JoinHandle<()>>,
    thread_id: Option<u32>,
}

impl InputListener for WindowsInputListener {
    fn start(&mut self, sink: EventSink) -> Result<(), CaptureError> {
        if self.thread.is_some() {
            return Err(CaptureError::Hook("InputListener läuft bereits".to_owned()));
        }
        {
            let mut guard = SINK
                .lock()
                .map_err(|_| CaptureError::Hook("Sink-Lock vergiftet".to_owned()))?;
            if guard.is_some() {
                return Err(CaptureError::Hook(
                    "es ist bereits ein InputListener in diesem Prozess aktiv".to_owned(),
                ));
            }
            *guard = Some(sink);
        }

        let (ready_tx, ready_rx) = mpsc::channel::<Result<u32, String>>();
        let thread = std::thread::Builder::new()
            .name("input-hooks".to_owned())
            .spawn(move || {
                // SAFETY: Hooks werden auf diesem Thread installiert und nach
                // Ende der Message-Loop wieder entfernt; die Callbacks sind
                // `extern "system"` mit korrekter Signatur.
                unsafe {
                    let mouse = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook), None, 0);
                    let keyboard = SetWindowsHookExW(WH_KEYBOARD_LL, Some(keyboard_hook), None, 0);
                    let (mouse, keyboard) = match (mouse, keyboard) {
                        (Ok(mouse), Ok(keyboard)) => (mouse, keyboard),
                        (mouse, keyboard) => {
                            if let Ok(hook) = mouse {
                                UnhookWindowsHookEx(hook).ok();
                            }
                            if let Ok(hook) = keyboard {
                                UnhookWindowsHookEx(hook).ok();
                            }
                            ready_tx
                                .send(Err("SetWindowsHookEx fehlgeschlagen".to_owned()))
                                .ok();
                            return;
                        }
                    };
                    ready_tx.send(Ok(GetCurrentThreadId())).ok();

                    let mut message = MSG::default();
                    while GetMessageW(&raw mut message, None, 0, 0).as_bool() {
                        let _ = TranslateMessage(&raw const message);
                        DispatchMessageW(&raw const message);
                    }

                    UnhookWindowsHookEx(mouse).ok();
                    UnhookWindowsHookEx(keyboard).ok();
                }
            })
            .map_err(|error| CaptureError::Hook(error.to_string()))?;

        match ready_rx.recv() {
            Ok(Ok(thread_id)) => {
                self.thread = Some(thread);
                self.thread_id = Some(thread_id);
                Ok(())
            }
            Ok(Err(message)) => {
                thread.join().ok();
                Self::clear_sink();
                Err(CaptureError::Hook(message))
            }
            Err(_) => {
                thread.join().ok();
                Self::clear_sink();
                Err(CaptureError::Hook(
                    "Hook-Thread unerwartet beendet".to_owned(),
                ))
            }
        }
    }

    fn stop(&mut self) -> Result<(), CaptureError> {
        if let Some(thread_id) = self.thread_id.take() {
            // SAFETY: Beendet die Message-Loop des Hook-Threads.
            unsafe {
                PostThreadMessageW(thread_id, WM_QUIT, WPARAM(0), LPARAM(0)).ok();
            }
        }
        if let Some(thread) = self.thread.take() {
            thread.join().ok();
        }
        Self::clear_sink();
        Ok(())
    }
}

impl WindowsInputListener {
    fn clear_sink() {
        if let Ok(mut guard) = SINK.lock() {
            *guard = None;
        }
    }
}

impl Drop for WindowsInputListener {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use windows::core::w;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DestroyWindow, GetDesktopWindow, HWND_MESSAGE, WINDOW_EX_STYLE,
        WINDOW_STYLE,
    };

    use super::*;

    /// Baut ein `mouseData`-Feld mit dem Delta im HIWORD.
    fn mouse_data(delta: i16) -> u32 {
        u32::from(u16::from_ne_bytes(delta.to_ne_bytes())) << 16
    }

    #[test]
    fn wheel_delta_dekodiert_hiword_vorzeichenbehaftet() {
        assert_eq!(wheel_delta(mouse_data(120)), 120);
        assert_eq!(wheel_delta(mouse_data(-360)), -360);
        // Touchpad-Bruchteil bleibt erhalten statt zu 0 zu werden.
        assert_eq!(wheel_delta(mouse_data(-15)), -15);
        assert_eq!(wheel_delta(0), 0);
    }

    #[test]
    fn system_double_click_threshold_ist_positiv() {
        let threshold = system_double_click_threshold();
        assert!(threshold.interval_ms > 0);
        assert!(threshold.width_px > 0 && threshold.height_px > 0);
    }

    #[test]
    fn modifier_vks_werden_erkannt() {
        for vk in [0x10, 0x11, 0x12, 0x5B, 0x5C, 0xA0, 0xA5] {
            assert!(is_modifier_vk(vk), "VK {vk:#04X} ist Modifier");
        }
        for vk in [0x41, 0x30, 0x70, 0x0D] {
            assert!(!is_modifier_vk(vk), "VK {vk:#04X} ist kein Modifier");
        }
    }

    fn combo_key(event: Option<RawEvent>) -> Option<String> {
        match event {
            Some(RawEvent::KeyCombo { combo, .. }) => Some(combo.key),
            _ => None,
        }
    }

    fn is_typing(event: Option<&RawEvent>) -> bool {
        matches!(event, Some(RawEvent::Typing { .. }))
    }

    #[test]
    fn altgr_tippt_ohne_tastenname() {
        // AltGr meldet Windows als Strg+Alt mit gedrückter rechter Alt-Taste.
        let altgr = Modifiers {
            ctrl: true,
            alt: true,
            altgr: true,
            ..Modifiers::default()
        };
        for vk in [0x51, 0x45] {
            // Q wird @, E wird € auf deutschem Layout
            assert!(
                is_typing(keyboard_event(vk, altgr, 1).as_ref()),
                "{vk:#04X}"
            );
        }
        let altgr_shift = Modifiers {
            shift: true,
            ..altgr
        };
        assert!(is_typing(keyboard_event(0xDB, altgr_shift, 1).as_ref()));
    }

    #[test]
    fn echte_kuerzel_bleiben_kombination() {
        let ctrl = Modifiers {
            ctrl: true,
            ..Modifiers::default()
        };
        assert_eq!(
            combo_key(keyboard_event(0x53, ctrl, 1)).as_deref(),
            Some("S")
        );
        let left_alt = Modifiers {
            alt: true,
            ..Modifiers::default()
        };
        assert_eq!(
            combo_key(keyboard_event(0x73, left_alt, 1)).as_deref(),
            Some("F4")
        );
        let win = Modifiers {
            win: true,
            ..Modifiers::default()
        };
        assert_eq!(
            combo_key(keyboard_event(0x45, win, 1)).as_deref(),
            Some("E")
        );
        let win_altgr = Modifiers { altgr: true, ..win };
        assert_eq!(
            combo_key(keyboard_event(0x45, win_altgr, 1)).as_deref(),
            Some("E")
        );
    }

    #[test]
    fn tasten_ohne_kuerzel_tippen_und_modifier_allein_zaehlen_nicht() {
        let shift = Modifiers {
            shift: true,
            ..Modifiers::default()
        };
        for modifiers in [Modifiers::default(), shift] {
            assert!(is_typing(keyboard_event(0x41, modifiers, 1).as_ref()));
        }
        let altgr = Modifiers {
            ctrl: true,
            alt: true,
            altgr: true,
            ..Modifiers::default()
        };
        for vk in [0x11, 0xA2, 0xA5, 0x10] {
            assert!(keyboard_event(vk, altgr, 1).is_none(), "{vk:#04X}");
        }
    }

    #[test]
    fn eigenes_fenster_wird_erkannt() {
        // SAFETY: Message-only-Fenster der Systemklasse STATIC, nur für
        // diesen Test angelegt und wieder zerstört.
        unsafe {
            let hwnd = CreateWindowExW(
                WINDOW_EX_STYLE(0),
                w!("STATIC"),
                w!("hows-test"),
                WINDOW_STYLE(0),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                None,
                None,
            )
            .expect("Fenster");
            assert!(is_own_process_window(hwnd));
            DestroyWindow(hwnd).expect("zerstören");
        }
    }

    #[test]
    fn own_process_window_lehnt_ungueltiges_hwnd_ab() {
        assert!(!is_own_process_window(HWND::default()));
    }

    #[test]
    fn own_process_window_lehnt_desktop_ab() {
        // Desktop-HWND gehört nicht zum Recorder-Prozess.
        let desktop = unsafe { GetDesktopWindow() };
        assert!(!is_own_process_window(desktop));
    }

    #[test]
    fn vk_namen_sind_kanonisch() {
        assert_eq!(vk_to_key_name(0x41), "A");
        assert_eq!(vk_to_key_name(0x39), "9");
        assert_eq!(vk_to_key_name(0x70), "F1");
        assert_eq!(vk_to_key_name(0x87), "F24");
        assert_eq!(vk_to_key_name(0x0D), "Enter");
        assert_eq!(vk_to_key_name(0x1B), "Esc");
        assert_eq!(vk_to_key_name(0x65), "Numpad5");
        // Layout-abhängige Tasten bekommen einen neutralen Namen.
        assert_eq!(vk_to_key_name(0xDB), "VK0xDB");
    }

    /// Smoke-Test auf der Windows-CI: Hooks installieren und sauber stoppen.
    #[test]
    fn listener_startet_und_stoppt() {
        let (tx, _rx) = mpsc::channel::<RawEvent>();
        let mut listener = WindowsInputListener::default();
        match listener.start(Box::new(move |event| {
            tx.send(event).ok();
        })) {
            Ok(()) => {
                // Doppelter Start wird abgelehnt.
                let (tx2, _rx2) = mpsc::channel::<RawEvent>();
                assert!(listener
                    .start(Box::new(move |event| {
                        tx2.send(event).ok();
                    }))
                    .is_err());
                listener.stop().expect("stop");
                assert!(SINK.lock().unwrap().is_none());
            }
            Err(error) => {
                eprintln!("Hooks nicht installierbar (Runner-Beschränkung?): {error}");
            }
        }
    }
}
