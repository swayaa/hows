//! Input-Hooks, Screenshots und Element-Inspektion.
//!
//! Aller OS-spezifischer Code liegt hinter den Traits [`InputListener`],
//! [`ScreenCapturer`] und [`ElementInspector`]; je Betriebssystem existiert
//! eine Implementierung (Windows in Meilenstein 3). Für Tests auf allen
//! Plattformen gibt es Fake-Implementierungen im Modul [`fake`].
//!
//! Privacy by design (Spec §5/§8): Tastatur-Events transportieren **nie**
//! Klartext. [`RawEvent::KeyCombo`] enthält nur Modifier-Zustand plus
//! Taste; reines Tippen wird nur als inhaltsloses [`RawEvent::Typing`]
//! gemeldet.

pub mod fake;

#[cfg(target_os = "windows")]
pub mod windows;

/// Punkt in physischen Pixeln des virtuellen Desktops (kann bei
/// Multi-Monitor-Setups negativ sein).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Point {
    /// X-Koordinate.
    pub x: i32,
    /// Y-Koordinate.
    pub y: i32,
}

/// Maustaste eines [`RawEvent::MouseDown`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    /// Linke Taste.
    Left,
    /// Rechte Taste.
    Right,
    /// Mittlere Taste (wird von der Session ignoriert, aber gemeldet).
    Middle,
}

/// Tastenkombination: Modifier-Zustand plus kanonischer Tastenname.
///
/// Der Tastenname ist die kanonische, sprachneutrale Bezeichnung
/// (z. B. `S`, `F5`, `Tab`), nie der getippte Text.
// Die vier Bools bilden die vier unabhängigen Modifier-Tasten 1:1 ab;
// eine Enum-Modellierung wäre hier künstlich.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyComboEvent {
    /// Strg gedrückt.
    pub ctrl: bool,
    /// Alt gedrückt.
    pub alt: bool,
    /// Umschalt gedrückt.
    pub shift: bool,
    /// Windows-Taste gedrückt.
    pub win: bool,
    /// Kanonischer Name der Haupttaste.
    pub key: String,
}

impl KeyComboEvent {
    /// `true`, wenn ein "echter" Modifier (Strg, Alt oder Win) beteiligt ist.
    ///
    /// Umschalt allein zählt nicht: `Shift+A` wäre Klartext-Tippen und darf
    /// nie als Schritt erfasst werden (Spec §5).
    pub fn has_real_modifier(&self) -> bool {
        self.ctrl || self.alt || self.win
    }

    /// Kanonische, sprachneutrale Darstellung
    /// (`Ctrl+Alt+Shift+Win+Taste` in fester Reihenfolge).
    pub fn canonical(&self) -> String {
        let mut parts: Vec<&str> = Vec::new();
        if self.ctrl {
            parts.push("Ctrl");
        }
        if self.alt {
            parts.push("Alt");
        }
        if self.shift {
            parts.push("Shift");
        }
        if self.win {
            parts.push("Win");
        }
        parts.push(&self.key);
        parts.join("+")
    }

    /// `true`, wenn diese Kombination dem konfigurierten Hotkey-String
    /// entspricht (z. B. `Ctrl+Shift+R`). Vergleich ohne Groß/Klein.
    pub fn matches_hotkey(&self, hotkey: &str) -> bool {
        self.canonical().eq_ignore_ascii_case(hotkey.trim())
    }
}

/// Rohes Input-Ereignis aus dem OS-Hook, Zeitstempel als Unix-Epoche in ms.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawEvent {
    /// Maustaste gedrückt (Screenshot-Auslöser, Spec §5).
    MouseDown {
        /// Position in physischen Pixeln.
        position: Point,
        /// Gedrückte Taste.
        button: MouseButton,
        /// Zeitpunkt in Unix-Millisekunden.
        timestamp_ms: u64,
    },
    /// Mausrad- oder Touchpad-Scroll.
    Wheel {
        /// Mausposition beim Scrollen.
        position: Point,
        /// Roh-Delta in Windows-Rad-Einheiten (120 = ein Raster); positiv =
        /// hoch bzw. rechts. Touchpads und hochauflösende Räder liefern
        /// Bruchteile eines Rasters.
        delta: i32,
        /// `true` bei horizontalem Scrollen.
        horizontal: bool,
        /// Zeitpunkt in Unix-Millisekunden.
        timestamp_ms: u64,
        /// Titel des Fensters unter dem Zeiger, von Win32, ohne UI Automation.
        window_title: Option<String>,
        /// Prozess-ID dieses Fensters. `0`, wenn Win32 keine liefert.
        window_pid: u32,
    },
    /// Nicht-Modifier-Taste gedrückt, mit aktuellem Modifier-Zustand.
    KeyCombo {
        /// Kombination (nie Text-Inhalt).
        combo: KeyComboEvent,
        /// Zeitpunkt in Unix-Millisekunden.
        timestamp_ms: u64,
    },
    /// Tipp-Aktivität ohne jeden Inhalt (für den zusammengefassten
    /// „Text eingegeben“-Schritt).
    Typing {
        /// Zeitpunkt in Unix-Millisekunden.
        timestamp_ms: u64,
    },
}

/// Monitor-Metadaten eines Screenshots (Per-Monitor-DPI, Spec §5).
#[derive(Debug, Clone, PartialEq)]
pub struct MonitorSnapshot {
    /// Monitor-Index im System.
    pub index: u32,
    /// X-Ursprung im virtuellen Desktop (physische Pixel).
    pub x: i32,
    /// Y-Ursprung im virtuellen Desktop (physische Pixel).
    pub y: i32,
    /// Breite in physischen Pixeln.
    pub width: u32,
    /// Höhe in physischen Pixeln.
    pub height: u32,
    /// DPI-Skalierungsfaktor (z. B. `1.5` für 150 %).
    pub scale_factor: f64,
}

/// Screenshot eines Monitors als PNG plus Monitor-Metadaten.
#[derive(Debug, Clone, PartialEq)]
pub struct Screenshot {
    /// PNG-kodierte Bilddaten (physische Pixel).
    pub png: Vec<u8>,
    /// Metadaten des aufgenommenen Monitors.
    pub monitor: MonitorSnapshot,
}

/// Metadaten des UI-Elements unter einem Punkt (UIA unter Windows).
///
/// Alle Felder sind optional: Bei Apps ohne Accessibility-Daten degradiert
/// die Aufnahme auf Position und Fenstertitel zurück (Spec §12).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CapturedElement {
    /// Element-Name (z. B. Button-Beschriftung).
    pub name: Option<String>,
    /// Kontrolltyp (z. B. `Button`, `MenuItem`).
    pub control_type: Option<String>,
    /// UIA-AutomationId.
    pub automation_id: Option<String>,
    /// Titel des Top-Level-Fensters.
    pub window_title: Option<String>,
    /// Name der Anwendung (Prozess).
    pub app_name: Option<String>,
}

/// Doppelklick-Schwellen: Zeitfenster und ein auf den ersten Klick
/// zentriertes Rechteck in physischen Pixeln (Windows-Semantik von
/// `GetDoubleClickTime` / `SM_CXDOUBLECLK` / `SM_CYDOUBLECLK`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DoubleClickThreshold {
    /// Maximaler Abstand zweier Klicks in ms.
    pub interval_ms: u64,
    /// Breite des Rechtecks.
    pub width_px: i32,
    /// Höhe des Rechtecks.
    pub height_px: i32,
}

impl Default for DoubleClickThreshold {
    /// Fallback, wenn das System keine Werte liefert (Nicht-Windows, Tests).
    fn default() -> Self {
        Self {
            interval_ms: 500,
            width_px: 16,
            height_px: 16,
        }
    }
}

impl DoubleClickThreshold {
    /// `true`, wenn der zweite Klick zeitlich und räumlich zum ersten passt.
    pub fn matches(&self, first: Point, first_ms: u64, second: Point, second_ms: u64) -> bool {
        second_ms.saturating_sub(first_ms) <= self.interval_ms
            && (second.x - first.x).abs() * 2 <= self.width_px
            && (second.y - first.y).abs() * 2 <= self.height_px
    }
}

/// Fehler in der Capture-Schicht.
#[derive(Debug, thiserror::Error)]
pub enum CaptureError {
    /// Screenshot konnte nicht erstellt werden.
    #[error("screenshot failed: {0}")]
    Screenshot(String),
    /// Element-Inspektion fehlgeschlagen.
    #[error("element inspection failed: {0}")]
    Inspection(String),
    /// Input-Hook konnte nicht installiert oder entfernt werden.
    #[error("input hook failed: {0}")]
    Hook(String),
}

/// Aktuelle Unix-Zeit in Millisekunden (Zeitstempel für [`RawEvent`]s).
pub fn now_ms() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| {
            u64::try_from(duration.as_millis()).unwrap_or(u64::MAX)
        })
}

/// Callback-Typ, über den ein [`InputListener`] Events meldet.
pub type EventSink = Box<dyn FnMut(RawEvent) + Send>;

/// Lauscht auf globale Maus-/Tastatur-Events (unter Windows via
/// `SetWindowsHookEx`, Spec §5) und meldet sie an einen Sink.
pub trait InputListener {
    /// Startet die Hooks; Events fließen in `sink`, bis [`stop`] gerufen wird.
    ///
    /// [`stop`]: InputListener::stop
    fn start(&mut self, sink: EventSink) -> Result<(), CaptureError>;

    /// Stoppt die Hooks. Idempotent.
    fn stop(&mut self) -> Result<(), CaptureError>;
}

/// Erstellt Screenshots des Monitors an einer gegebenen Position.
pub trait ScreenCapturer {
    /// Screenshot des Monitors, der `position` enthält (Fallback: primärer
    /// Monitor). Der Hook reicht den Mausdruck nur an den Recorder-Thread
    /// weiter, dort läuft dieser Aufruf danach: Das Bild entsteht um den
    /// Mausdruck herum, die Oberfläche kann schon reagiert haben.
    fn capture_at(&mut self, position: Point) -> Result<Screenshot, CaptureError>;
}

/// Ermittelt UI-Element-Metadaten (Hit-Test bzw. Fokus).
pub trait ElementInspector {
    /// Element unter dem Punkt (UIA-Hit-Test).
    fn element_at(&mut self, position: Point) -> Result<CapturedElement, CaptureError>;

    /// Aktuell fokussiertes Element (für zusammengefasste Texteingabe).
    fn focused_element(&mut self) -> Result<CapturedElement, CaptureError>;
}

#[cfg(test)]
mod tests {
    use super::{DoubleClickThreshold, KeyComboEvent, Point};

    #[test]
    fn double_click_rectangle_is_centered_on_first_click() {
        // Windows-Default bei 96 DPI. 500 ms, 4 mal 4 px, also 2 px um den Klick.
        let threshold = DoubleClickThreshold {
            interval_ms: 500,
            width_px: 4,
            height_px: 4,
        };
        let first = Point { x: 100, y: 100 };
        assert!(threshold.matches(first, 1000, Point { x: 102, y: 98 }, 1500));
        assert!(!threshold.matches(first, 1000, Point { x: 103, y: 100 }, 1100));
        assert!(!threshold.matches(first, 1000, first, 1501));
    }

    #[test]
    fn key_combo_canonical_und_hotkey_match() {
        let combo = KeyComboEvent {
            ctrl: true,
            alt: false,
            shift: true,
            win: false,
            key: "R".to_owned(),
        };
        assert_eq!(combo.canonical(), "Ctrl+Shift+R");
        assert!(combo.matches_hotkey("Ctrl+Shift+R"));
        assert!(combo.matches_hotkey("ctrl+shift+r"));
        assert!(!combo.matches_hotkey("Ctrl+Shift+P"));
    }
}
