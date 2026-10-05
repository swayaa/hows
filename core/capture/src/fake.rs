//! Fake-Implementierungen der Capture-Traits für Tests auf allen Plattformen.
//!
//! Die Session-, Store- und Export-Tests laufen gegen diese Fakes; der echte
//! Windows-Code wird von der Windows-CI kompiliert und getestet.

use std::collections::VecDeque;

use crate::{
    CaptureError, CapturedElement, ElementInspector, EventSink, InputListener, MonitorSnapshot,
    Point, RawEvent, ScreenCapturer, Screenshot,
};

/// Platzhalter-PNG-Bytes der Fakes (kein echtes Bild, nur Signatur + Füllung).
pub const FAKE_PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0xFA, 0x4B];

/// Fake-[`ScreenCapturer`]: liefert konfigurierbare Screenshots und
/// protokolliert alle Aufrufe.
#[derive(Debug)]
pub struct FakeScreenCapturer {
    /// Monitor-Metadaten, die jeder Screenshot trägt.
    pub monitor: MonitorSnapshot,
    /// PNG-Bytes, die jeder Screenshot trägt.
    pub png: Vec<u8>,
    /// Positionen aller bisherigen `capture_at`-Aufrufe.
    pub captured_at: Vec<Point>,
    /// Wenn gesetzt, schlägt der nächste Aufruf mit diesem Fehler fehl.
    pub fail_next: Option<String>,
}

impl Default for FakeScreenCapturer {
    fn default() -> Self {
        Self {
            monitor: MonitorSnapshot {
                index: 0,
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
                scale_factor: 1.0,
            },
            png: FAKE_PNG.to_vec(),
            captured_at: Vec::new(),
            fail_next: None,
        }
    }
}

impl ScreenCapturer for FakeScreenCapturer {
    fn capture_at(&mut self, position: Point) -> Result<Screenshot, CaptureError> {
        if let Some(message) = self.fail_next.take() {
            return Err(CaptureError::Screenshot(message));
        }
        self.captured_at.push(position);
        Ok(Screenshot {
            png: self.png.clone(),
            monitor: self.monitor.clone(),
        })
    }
}

/// Fake-[`ElementInspector`]: liefert Elemente aus einer Warteschlange,
/// danach ein konfigurierbares Default-Element.
#[derive(Debug, Default)]
pub struct FakeElementInspector {
    /// Elemente, die der Reihe nach von `element_at` geliefert werden.
    pub queue: VecDeque<CapturedElement>,
    /// Default-Element, wenn die Warteschlange leer ist.
    pub default_element: CapturedElement,
    /// Element, das `focused_element` liefert.
    pub focused: CapturedElement,
    /// Positionen aller bisherigen `element_at`-Aufrufe.
    pub inspected_at: Vec<Point>,
}

impl FakeElementInspector {
    /// Bequemer Konstruktor: alle Abfragen liefern dasselbe Element.
    pub fn returning(element: CapturedElement) -> Self {
        Self {
            queue: VecDeque::new(),
            focused: element.clone(),
            default_element: element,
            inspected_at: Vec::new(),
        }
    }
}

impl ElementInspector for FakeElementInspector {
    fn element_at(&mut self, position: Point) -> Result<CapturedElement, CaptureError> {
        self.inspected_at.push(position);
        Ok(self
            .queue
            .pop_front()
            .unwrap_or_else(|| self.default_element.clone()))
    }

    fn focused_element(&mut self) -> Result<CapturedElement, CaptureError> {
        Ok(self.focused.clone())
    }
}

/// Fake-[`InputListener`]: hält den Sink und erlaubt Tests, Events
/// einzuspeisen, als kämen sie vom OS-Hook.
#[derive(Default)]
pub struct FakeInputListener {
    sink: Option<EventSink>,
    /// Wenn gesetzt, schlägt der nächste `start` mit diesem Fehler fehl.
    pub fail_next: Option<String>,
}

impl FakeInputListener {
    /// Speist ein Event in den registrierten Sink ein.
    ///
    /// # Panics
    ///
    /// Wenn der Listener nicht gestartet ist.
    pub fn emit(&mut self, event: RawEvent) {
        let sink = self
            .sink
            .as_mut()
            .expect("FakeInputListener: start() wurde nicht aufgerufen");
        sink(event);
    }

    /// `true`, wenn der Listener gestartet ist.
    pub fn is_running(&self) -> bool {
        self.sink.is_some()
    }
}

impl InputListener for FakeInputListener {
    fn start(&mut self, sink: EventSink) -> Result<(), CaptureError> {
        if let Some(message) = self.fail_next.take() {
            return Err(CaptureError::Hook(message));
        }
        self.sink = Some(sink);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), CaptureError> {
        self.sink = None;
        Ok(())
    }
}
