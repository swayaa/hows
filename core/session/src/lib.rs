//! Aufnahme-Statemachine (`Idle`, `Recording`, `Paused`, `Reviewing`) und
//! Zusammenbau der Schritte (Spec §3, §5).
//!
//! Die Session nimmt [`RawEvent`]s vom `InputListener` und baut daraus
//! Schritte im Datenmodell von `steps-store`. Dazu gehören der Screenshot
//! beim Mausdruck, die Element-Daten, die Texte aus `steps-i18n`, die
//! Scroll-Zusammenfassung und die Doppelklick-Erkennung.
//!
//! Tastatur nie im Klartext. Nur Kombinationen mit echtem Modifier
//! (Strg, Alt oder Win) werden ein Schritt. Alles andere wird ein
//! Texteingabe-Schritt ohne Inhalt.

mod texts;

pub use steps_i18n::Locale;
pub use texts::{app_name, derive_guide_title, guide_title_from, ClickKind, Stamp};

use std::collections::BTreeMap;

use steps_capture::{
    CapturedElement, DoubleClickThreshold, ElementInspector, KeyComboEvent, MouseButton, Point,
    RawEvent, ScreenCapturer, Screenshot,
};
use steps_store::{
    Action, ElementInfo, Guide, Monitor, Position, ScrollDirection, Step, StepText, StepsFile,
};

/// Standard der Tipp-Pause (ms), siehe [`Pauses::typing_ms`].
pub const TYPING_GAP_MS: u64 = 3000;
/// Standard der Scroll-Pause (ms), siehe [`Pauses::scroll_ms`].
pub const SCROLL_GAP_MS: u64 = 1500;

/// Pausen, nach denen eine Zusammenfassung endet und ein neuer Schritt beginnt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pauses {
    /// Maximale Tipp-Pause (ms) innerhalb einer Texteingabe.
    pub typing_ms: u64,
    /// Maximale Scroll-Pause (ms) innerhalb eines Scroll-Schritts.
    pub scroll_ms: u64,
}

impl Default for Pauses {
    fn default() -> Self {
        Self {
            typing_ms: TYPING_GAP_MS,
            scroll_ms: SCROLL_GAP_MS,
        }
    }
}
/// Ein Rad-Raster in Roh-Delta-Einheiten (Windows `WHEEL_DELTA`).
pub const WHEEL_NOTCH: u32 = 120;

/// Zustand der Aufnahme-Statemachine (Spec §3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    /// Keine Aufnahme aktiv.
    Idle,
    /// Aufnahme läuft, Events werden zu Schritten verarbeitet.
    Recording,
    /// Aufnahme pausiert, Events werden ignoriert.
    Paused,
    /// Aufnahme beendet, Schritte bereit für den Editor.
    Reviewing,
}

/// Fehler der Session-Schicht. Es sind Programmierfehler; die App meldet sie
/// als Code `internal`, der Text ist ein englisches Detail für Logs.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    /// Der angeforderte Zustandsübergang ist nicht erlaubt.
    #[error("invalid state change from {from:?} to {to:?}")]
    InvalidTransition {
        /// Ausgangszustand.
        from: SessionState,
        /// Angefragter Zielzustand.
        to: SessionState,
    },
    /// Guide-Erzeugung ist nur im Zustand `Reviewing` möglich.
    #[error("a guide can only be built while reviewing (state is {state:?})")]
    NotReviewing {
        /// Tatsächlicher Zustand.
        state: SessionState,
    },
}

/// Ergebnis der Event-Verarbeitung: welcher Schritt entstand oder sich
/// änderte (Index in [`Session::steps`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepChange {
    /// Neuer Schritt am Index.
    Created(usize),
    /// Bestehender Schritt am Index wurde aktualisiert (z. B. Scroll-Menge
    /// erhöht oder Klick zu Doppelklick zusammengeführt).
    Updated(usize),
}

/// Laufende Scroll-Zusammenfassung: ein Schritt pro Richtung und Fenster
/// (Spec §5). Das Fenster ist Titel plus Prozess-ID von Win32. Das Element
/// unter dem Zeiger wechselt, während der Inhalt durchläuft, und zählt nicht.
struct ScrollGroup {
    direction: ScrollDirection,
    window: (Option<String>, u32),
    step_index: usize,
    last_timestamp_ms: u64,
    raw_total: u32,
}

/// Fenstertitel ohne Leerzeichen an den Enden. Ein leerer Titel ist keiner.
fn window_key(title: Option<String>, pid: u32) -> (Option<String>, u32) {
    let title = title.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    });
    (title, pid)
}

/// Merker für die Doppelklick-Erkennung.
struct LastClick {
    position: Point,
    timestamp_ms: u64,
    step_index: usize,
}

/// Laufende Texteingabe-Zusammenfassung.
struct TypingGroup {
    last_timestamp_ms: u64,
    step_index: usize,
}

/// Aufnahme-Session gegen die Capture-Traits (echte Implementierungen unter
/// Windows, Fakes in Tests).
pub struct Session<C: ScreenCapturer, I: ElementInspector> {
    capturer: C,
    inspector: I,
    locale: Locale,
    state: SessionState,
    steps: Vec<Step>,
    images: BTreeMap<String, Vec<u8>>,
    next_step_number: u32,
    scroll_group: Option<ScrollGroup>,
    last_click: Option<LastClick>,
    typing_group: Option<TypingGroup>,
    last_mouse_position: Point,
    double_click: DoubleClickThreshold,
    pauses: Pauses,
}

fn to_element_info(element: CapturedElement) -> ElementInfo {
    ElementInfo {
        name: element.name,
        control_type: element.control_type,
        automation_id: element.automation_id,
        window_title: element.window_title,
        app_name: element.app_name,
        ..ElementInfo::default()
    }
}

fn to_position(point: Point) -> Position {
    Position {
        x: point.x,
        y: point.y,
    }
}

fn to_monitor(screenshot: &Screenshot) -> Monitor {
    Monitor {
        index: screenshot.monitor.index,
        x: screenshot.monitor.x,
        y: screenshot.monitor.y,
        width: screenshot.monitor.width,
        height: screenshot.monitor.height,
        scale_factor: screenshot.monitor.scale_factor,
    }
}

impl<C: ScreenCapturer, I: ElementInspector> Session<C, I> {
    /// Erzeugt eine neue Session im Zustand `Idle`.
    pub fn new(capturer: C, inspector: I, locale: Locale) -> Self {
        Self {
            capturer,
            inspector,
            locale,
            state: SessionState::Idle,
            steps: Vec::new(),
            images: BTreeMap::new(),
            next_step_number: 1,
            scroll_group: None,
            last_click: None,
            typing_group: None,
            last_mouse_position: Point::default(),
            double_click: DoubleClickThreshold::default(),
            pauses: Pauses::default(),
        }
    }

    /// Setzt die Doppelklick-Schwellen (unter Windows aus den
    /// Systemeinstellungen, siehe `steps_capture::windows`).
    #[must_use]
    pub fn with_double_click(mut self, threshold: DoubleClickThreshold) -> Self {
        self.double_click = threshold;
        self
    }

    /// Setzt die Pausen für neue Zusammenfassungen, siehe [`Session::set_pauses`].
    #[must_use]
    pub fn with_pauses(mut self, pauses: Pauses) -> Self {
        self.pauses = pauses;
        self
    }

    /// Ändert die Pausen, auch während einer Aufnahme. Sie gelten ab dem
    /// nächsten Event.
    pub fn set_pauses(&mut self, pauses: Pauses) {
        self.pauses = pauses;
    }

    /// Aktueller Zustand.
    pub fn state(&self) -> SessionState {
        self.state
    }

    /// Sprache der generierten Texte.
    pub fn locale(&self) -> Locale {
        self.locale
    }

    /// Bisher aufgenommene Schritte.
    pub fn steps(&self) -> &[Step] {
        &self.steps
    }

    /// Screenshots je Schritt-ID.
    pub fn images(&self) -> &BTreeMap<String, Vec<u8>> {
        &self.images
    }

    /// Von Idle nach Recording.
    pub fn start(&mut self) -> Result<(), SessionError> {
        self.transition(SessionState::Idle, SessionState::Recording)
    }

    /// Von Recording nach Paused.
    pub fn pause(&mut self) -> Result<(), SessionError> {
        self.flush_aggregation();
        self.transition(SessionState::Recording, SessionState::Paused)
    }

    /// Von Paused nach Recording.
    pub fn resume(&mut self) -> Result<(), SessionError> {
        self.transition(SessionState::Paused, SessionState::Recording)
    }

    /// Von Recording oder Paused nach Reviewing.
    pub fn stop(&mut self) -> Result<(), SessionError> {
        match self.state {
            SessionState::Recording | SessionState::Paused => {
                self.flush_aggregation();
                self.state = SessionState::Reviewing;
                Ok(())
            }
            from => Err(SessionError::InvalidTransition {
                from,
                to: SessionState::Reviewing,
            }),
        }
    }

    /// Verwirft die Aufnahme und kehrt zu `Idle` zurück.
    pub fn discard(&mut self) {
        self.steps.clear();
        self.images.clear();
        self.next_step_number = 1;
        self.flush_aggregation();
        self.state = SessionState::Idle;
    }

    fn transition(&mut self, from: SessionState, to: SessionState) -> Result<(), SessionError> {
        if self.state == from {
            self.state = to;
            Ok(())
        } else {
            Err(SessionError::InvalidTransition {
                from: self.state,
                to,
            })
        }
    }

    /// Beendet laufende Zusammenfassungen (Scroll, Tippen, Doppelklick).
    fn flush_aggregation(&mut self) {
        self.scroll_group = None;
        self.typing_group = None;
        self.last_click = None;
    }

    /// Verarbeitet ein Input-Event. Außerhalb von `Recording` werden Events
    /// ignoriert.
    pub fn handle_event(&mut self, event: RawEvent) -> Option<StepChange> {
        if self.state != SessionState::Recording {
            return None;
        }
        match event {
            RawEvent::MouseDown {
                position,
                button,
                timestamp_ms,
            } => self.on_mouse_down(position, button, timestamp_ms),
            RawEvent::Wheel {
                position,
                delta,
                horizontal,
                timestamp_ms,
                window_title,
                window_pid,
            } => self.on_wheel(
                position,
                delta,
                horizontal,
                timestamp_ms,
                window_title,
                window_pid,
            ),
            RawEvent::KeyCombo {
                combo,
                timestamp_ms,
            } => Some(self.on_key_combo(&combo, timestamp_ms)),
            RawEvent::Typing { timestamp_ms } => Some(self.on_typing(timestamp_ms)),
        }
    }

    /// Fügt einen manuellen Notiz-Schritt hinzu (in `Recording` und
    /// `Reviewing` erlaubt).
    pub fn add_manual_note(
        &mut self,
        text: impl Into<String>,
        timestamp_ms: u64,
    ) -> Option<StepChange> {
        if !matches!(
            self.state,
            SessionState::Recording | SessionState::Reviewing
        ) {
            return None;
        }
        self.flush_aggregation();
        let index = self.push_step(
            Action::ManualNote,
            timestamp_ms,
            None,
            None,
            None,
            text.into(),
        );
        Some(StepChange::Created(index))
    }

    /// Erzeugt aus der abgeschlossenen Session eine `.steps`-Struktur.
    pub fn into_steps_file(
        self,
        title: impl Into<String>,
        created_at_ms: u64,
    ) -> Result<StepsFile, SessionError> {
        if self.state != SessionState::Reviewing {
            return Err(SessionError::NotReviewing { state: self.state });
        }
        let mut guide = Guide::new(title, created_at_ms, self.locale.code());
        guide.steps = self.steps;
        Ok(StepsFile {
            guide,
            images: self.images,
        })
    }

    fn on_mouse_down(
        &mut self,
        position: Point,
        button: MouseButton,
        timestamp_ms: u64,
    ) -> Option<StepChange> {
        self.scroll_group = None;
        self.typing_group = None;
        self.last_mouse_position = position;

        let kind = match button {
            MouseButton::Left => ClickKind::Single,
            MouseButton::Right => ClickKind::Right,
            // Mittlere Taste ist kein Aktionstyp des Schemas (Spec §4).
            MouseButton::Middle => {
                self.last_click = None;
                return None;
            }
        };

        // Doppelklick-Erkennung: Der Low-Level-Hook liefert keine
        // Doppelklick-Nachrichten, daher werden zwei nahe Linksklicks hier
        // zusammengeführt. Der Screenshot des ersten Klicks bleibt erhalten.
        if kind == ClickKind::Single {
            if let Some(last) = &self.last_click {
                let index = last.step_index;
                if self.double_click.matches(
                    last.position,
                    last.timestamp_ms,
                    position,
                    timestamp_ms,
                ) && matches!(self.steps[index].action, Action::Click)
                {
                    self.steps[index].action = Action::DoubleClick;
                    let element = self.steps[index].element.clone().unwrap_or_default();
                    self.steps[index].text =
                        StepText::generated(texts::click(self.locale, ClickKind::Double, &element));
                    // Ein dritter Klick startet eine neue Erkennung.
                    self.last_click = None;
                    return Some(StepChange::Updated(index));
                }
            }
        }

        // Erst der Screenshot um den Mausdruck, dann die Element-Abfrage.
        // Fehlt der Screenshot, bleibt der Schritt ohne Bild.
        let screenshot = self.capturer.capture_at(position).ok();
        let element = self.inspector.element_at(position).unwrap_or_default();
        let element_info = to_element_info(element);

        let action = match kind {
            ClickKind::Single => Action::Click,
            ClickKind::Right => Action::RightClick,
            ClickKind::Double => unreachable!("Doppelklick entsteht nur durch Zusammenführung"),
        };
        let text = texts::click(self.locale, kind, &element_info);
        let index = self.push_step(
            action,
            timestamp_ms,
            Some(to_position(position)),
            screenshot,
            Some(element_info),
            text,
        );

        self.last_click = if kind == ClickKind::Single {
            Some(LastClick {
                position,
                timestamp_ms,
                step_index: index,
            })
        } else {
            None
        };
        Some(StepChange::Created(index))
    }

    fn on_wheel(
        &mut self,
        position: Point,
        delta: i32,
        horizontal: bool,
        timestamp_ms: u64,
        window_title: Option<String>,
        window_pid: u32,
    ) -> Option<StepChange> {
        self.typing_group = None;
        self.last_click = None;
        self.last_mouse_position = position;
        if delta == 0 {
            return None;
        }

        let direction = match (horizontal, delta > 0) {
            (false, true) => ScrollDirection::Up,
            (false, false) => ScrollDirection::Down,
            (true, false) => ScrollDirection::Left,
            (true, true) => ScrollDirection::Right,
        };
        let raw = delta.unsigned_abs();
        let window = window_key(window_title, window_pid);

        // Gleiche Richtung, gleiches Fenster, Pause nicht um: nur die Menge.
        // UI Automation und Screenshot nur beim ersten Tick der Gruppe.
        let continued = self.scroll_group.as_mut().and_then(|group| {
            let open = group.direction == direction
                && group.window == window
                && timestamp_ms.saturating_sub(group.last_timestamp_ms) <= self.pauses.scroll_ms;
            if !open {
                return None;
            }
            group.last_timestamp_ms = timestamp_ms;
            group.raw_total = group.raw_total.saturating_add(raw);
            Some((group.step_index, group.raw_total.div_ceil(WHEEL_NOTCH)))
        });
        if let Some((index, amount)) = continued {
            self.write_scroll_amount(index, direction, amount);
            return Some(StepChange::Updated(index));
        }

        let screenshot = self.capturer.capture_at(position).ok();
        let element = self.inspector.element_at(position).unwrap_or_default();
        let element_info = to_element_info(element);
        let amount = raw.div_ceil(WHEEL_NOTCH);
        let text = texts::scroll(self.locale, direction, &element_info, amount);
        let index = self.push_step(
            Action::Scroll { direction, amount },
            timestamp_ms,
            Some(to_position(position)),
            screenshot,
            Some(element_info),
            text,
        );
        self.scroll_group = Some(ScrollGroup {
            direction,
            window,
            step_index: index,
            last_timestamp_ms: timestamp_ms,
            raw_total: raw,
        });
        Some(StepChange::Created(index))
    }

    fn write_scroll_amount(&mut self, index: usize, direction: ScrollDirection, amount: u32) {
        let step = &mut self.steps[index];
        if let Action::Scroll { amount: slot, .. } = &mut step.action {
            *slot = amount;
        }
        let element = step.element.clone().unwrap_or_default();
        step.text.generated = texts::scroll(self.locale, direction, &element, amount);
    }

    fn on_key_combo(&mut self, combo: &KeyComboEvent, timestamp_ms: u64) -> StepChange {
        // Key-Combo-Regel (Spec §5): nie Klartext. Ohne echten Modifier
        // (Strg, Alt oder Win) wird die Taste verworfen und nur als
        // Tipp-Aktivität ohne Inhalt gewertet. Umschalt allein zählt nicht.
        if !combo.has_real_modifier() {
            return self.on_typing(timestamp_ms);
        }

        self.flush_aggregation();
        let screenshot = self.capturer.capture_at(self.last_mouse_position).ok();
        let element = self.inspector.focused_element().unwrap_or_default();
        let element_info = to_element_info(element);
        let canonical = combo.canonical();
        let text = texts::key_combo(self.locale, &canonical);
        let index = self.push_step(
            Action::KeyCombo { combo: canonical },
            timestamp_ms,
            None,
            screenshot,
            Some(element_info),
            text,
        );
        StepChange::Created(index)
    }

    fn on_typing(&mut self, timestamp_ms: u64) -> StepChange {
        self.scroll_group = None;
        self.last_click = None;

        if let Some(group) = &mut self.typing_group {
            if timestamp_ms.saturating_sub(group.last_timestamp_ms) <= self.pauses.typing_ms {
                group.last_timestamp_ms = timestamp_ms;
                return StepChange::Updated(group.step_index);
            }
        }

        let element = self.inspector.focused_element().unwrap_or_default();
        let element_info = to_element_info(element);
        let text = texts::text_input(self.locale, &element_info);
        // Bewusst ohne Screenshot: Nach einer Tipp-Pause zeigte er den schon
        // getippten Text und unterliefe „Tastatur nie im Klartext“. Das Feld
        // zeigt meist der vorangehende Klick-Schritt.
        let index = self.push_step(
            Action::TextInput,
            timestamp_ms,
            None,
            None,
            Some(element_info),
            text,
        );
        self.typing_group = Some(TypingGroup {
            last_timestamp_ms: timestamp_ms,
            step_index: index,
        });
        StepChange::Created(index)
    }

    fn push_step(
        &mut self,
        action: Action,
        timestamp_ms: u64,
        position: Option<Position>,
        screenshot: Option<Screenshot>,
        element: Option<ElementInfo>,
        text: String,
    ) -> usize {
        let id = format!("step-{:04}", self.next_step_number);
        self.next_step_number += 1;

        let monitor = screenshot.as_ref().map(to_monitor);
        if let Some(shot) = screenshot {
            self.images.insert(id.clone(), shot.png);
        }

        self.steps.push(Step {
            id,
            action,
            timestamp_ms,
            position,
            monitor,
            element,
            text: StepText::generated(text),
            overlays: Vec::new(),
            crop: None,
        });
        self.steps.len() - 1
    }
}
