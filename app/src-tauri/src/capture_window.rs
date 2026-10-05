//! Hauptfenster während einer Aufnahme: kompakte Pill oben mittig, immer im
//! Vordergrund und von Screenshots ausgenommen. Danach kehrt das Fenster in
//! seine vorherige Größe und Position zurück.
//!
//! Ein Hotkey startet oft, während Hows versteckt, minimiert oder ohne Fokus
//! ist. Das Fenster darf dann nicht erst in voller Größe ohne Rahmen
//! erscheinen und nicht den Fokus der App klauen, die gerade aufgenommen wird.

use std::sync::Mutex;

use tauri::{AppHandle, LogicalSize, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};

use crate::error::{CommandError, ErrorCode};

/// Abstand der Pill zum oberen Rand des Bildschirms, in logischen Pixeln.
const TOP_MARGIN: f64 = 12.0;

/// Erste Größe, bevor die Oberfläche die echte Pill misst. Breit genug, dass
/// der Inhalt nicht abgeschnitten wird, und klein genug, dass kein Vollfenster
/// aufblitzt. Die Smoke-Prüfung erwartet unter 800×120.
const DEFAULT_PILL_WIDTH: f64 = 720.0;
const DEFAULT_PILL_HEIGHT: f64 = 96.0;

/// Größer als das gilt die Messung als Vollfenster, nicht als Pill.
const MAX_PILL_WIDTH: f64 = 800.0;
const MAX_PILL_HEIGHT: f64 = 160.0;

/// Windows parkt minimierte Fenster bei ungefähr −32000.
const OFF_SCREEN: i32 = -16_000;

const DEFAULT_WINDOW_WIDTH: f64 = 1100.0;
const DEFAULT_WINDOW_HEIGHT: f64 = 750.0;

#[derive(Debug, Clone, Copy)]
struct Geometry {
    position: PhysicalPosition<i32>,
    size: PhysicalSize<u32>,
    maximized: bool,
}

#[derive(Debug, Default)]
struct Compact {
    /// Fenster vor der Aufnahme; `Some` heißt, die Pill ist aktiv.
    before: Option<Geometry>,
    /// Die Pill wurde einmal oben mittig platziert. Danach bleibt sie dort,
    /// wo der Nutzer sie hingezogen hat.
    placed: bool,
}

/// Zustand des Aufnahmefensters, von Tauri verwaltet.
#[derive(Debug, Default)]
pub struct CaptureWindow(Mutex<Compact>);

impl CaptureWindow {
    /// `true`, solange die kompakte Aufnahme-Pill aktiv ist.
    pub(crate) fn active(&self) -> bool {
        self.0
            .lock()
            .map(|compact| compact.before.is_some())
            .unwrap_or(false)
    }
}

fn main_window(app: &AppHandle) -> Option<WebviewWindow> {
    app.get_webview_window("main")
}

/// Passt das Fenster an einen Recorder-Zustand an.
pub(crate) fn sync(app: &AppHandle, state: &str) {
    let (Some(window), Some(capture)) = (main_window(app), app.try_state::<CaptureWindow>()) else {
        return;
    };
    let Ok(mut compact) = capture.0.lock() else {
        return;
    };
    if matches!(state, "recording" | "paused") {
        enter(&window, &mut compact);
    } else {
        leave(&window, &mut compact);
    }
}

/// Zweiter Start während der Aufnahme: die Pill auf den Bildschirm holen,
/// ohne sie zum Vollfenster zu machen und ohne den Fokus zu übernehmen.
pub(crate) fn reveal(app: &AppHandle) {
    let Some(window) = main_window(app) else {
        return;
    };
    let off = window
        .outer_position()
        .ok()
        .is_none_or(|position| !position_on_screen(position));
    if off {
        apply_pill_bounds(&window, DEFAULT_PILL_WIDTH, DEFAULT_PILL_HEIGHT, true).ok();
    }
    present_without_focus(&window);
}

fn enter(window: &WebviewWindow, compact: &mut Compact) {
    if compact.before.is_some() {
        return;
    }
    // Erst zurückholen, sonst meldet Windows die Parkposition −32000 und
    // ein danach gesetztes Maß geht beim Wiederherstellen verloren.
    if window.is_minimized().unwrap_or(false) {
        window.unminimize().ok();
    }
    let maximized = window.is_maximized().unwrap_or(false);
    let saved = match (window.outer_position(), window.inner_size()) {
        (Ok(position), Ok(size)) if restore_is_usable(position, size) => Geometry {
            position,
            size,
            maximized,
        },
        _ => default_geometry(window),
    };
    compact.before = Some(saved);
    compact.placed = false;
    // Zuerst aus Screenshots nehmen, damit schon der erste Schritt sauber ist.
    window.set_content_protected(true).ok();
    if maximized {
        window.unmaximize().ok();
    }
    window.set_decorations(false).ok();
    window.set_resizable(false).ok();
    window.set_always_on_top(true).ok();
    // Größe setzen, bevor das Fenster sichtbar wird. Sonst blitzt das
    // volle Hows-Fenster ohne Rahmen auf und die Pill wirkt verzogen.
    apply_pill_bounds(window, DEFAULT_PILL_WIDTH, DEFAULT_PILL_HEIGHT, true).ok();
    present_without_focus(window);
}

fn leave(window: &WebviewWindow, compact: &mut Compact) {
    let Some(before) = compact.before.take() else {
        return;
    };
    compact.placed = false;
    window.set_always_on_top(false).ok();
    window.set_resizable(true).ok();
    window.set_decorations(true).ok();
    window.set_size(before.size).ok();
    window.set_position(before.position).ok();
    if before.maximized {
        window.maximize().ok();
    }
    window.set_content_protected(false).ok();
    // Nach Stop den Editor wieder zeigen, auch wenn der Start aus dem Tray kam.
    window.unminimize().ok();
    window.show().ok();
    window.set_focus().ok();
}

/// Sichtbar machen, ohne die App im Vordergrund zur Aufnahme zu machen.
fn present_without_focus(window: &WebviewWindow) {
    if window.is_visible().unwrap_or(false) && !window.is_minimized().unwrap_or(false) {
        return;
    }
    show_no_activate(window);
}

#[cfg(windows)]
fn show_no_activate(window: &WebviewWindow) {
    use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_SHOWNOACTIVATE};
    let Ok(hwnd) = window.hwnd() else {
        return;
    };
    // Der Handle gehört zu diesem Fenster. SW_SHOWNOACTIVATE zeigt es, ohne
    // SetForegroundWindow aufzurufen.
    unsafe {
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);
    }
}

#[cfg(not(windows))]
fn show_no_activate(window: &WebviewWindow) {
    window.show().ok();
}

/// Holt das volle Fenster nach vorn, damit der Rückfrage-Dialog Platz hat.
/// `before` bleibt gesetzt, die Pill kommt mit `restore_pill` zurück.
pub(crate) fn open_dialog_host(app: &AppHandle) {
    let Some(window) = main_window(app) else {
        return;
    };
    let geometry = app
        .try_state::<CaptureWindow>()
        .and_then(|capture| capture.0.lock().ok().and_then(|compact| compact.before));
    let Some(before) = geometry else {
        window.unminimize().ok();
        window.show().ok();
        window.set_focus().ok();
        return;
    };
    window.set_always_on_top(false).ok();
    window.set_resizable(true).ok();
    window.set_decorations(true).ok();
    window.set_size(before.size).ok();
    window.set_position(before.position).ok();
    if before.maximized {
        window.maximize().ok();
    }
    window.unminimize().ok();
    window.show().ok();
    window.set_focus().ok();
}

/// Setzt die Aufnahme-Pill zurück, nachdem die Rückfrage abgebrochen wurde.
pub(crate) fn restore_pill(app: &AppHandle) {
    let Some(window) = main_window(app) else {
        return;
    };
    let Some(capture) = app.try_state::<CaptureWindow>() else {
        return;
    };
    let Ok(mut compact) = capture.0.lock() else {
        return;
    };
    if compact.before.is_none() {
        return;
    }
    compact.placed = false;
    drop(compact);
    if window.is_maximized().unwrap_or(false) {
        window.unmaximize().ok();
    }
    window.set_always_on_top(true).ok();
    window.set_resizable(false).ok();
    window.set_decorations(false).ok();
    apply_pill_bounds(&window, DEFAULT_PILL_WIDTH, DEFAULT_PILL_HEIGHT, true).ok();
}

/// Zeigt das Fenster groß genug für den bestehenden Bestätigungsdialog.
#[tauri::command]
pub fn present_for_dialog(app: AppHandle) {
    open_dialog_host(&app);
}

/// Bringt die Pill zurück, wenn die Rückfrage abgebrochen wurde.
#[tauri::command]
pub fn restore_capture_pill(app: AppHandle) {
    restore_pill(&app);
}

/// Die Oberfläche meldet die gemessene Größe der Pill. Beim ersten Mal wird
/// sie oben mittig auf dem aktuellen Bildschirm platziert.
#[tauri::command]
pub fn fit_capture_window(app: AppHandle, width: f64, height: f64) -> Result<(), CommandError> {
    let window = main_window(&app)
        .ok_or_else(|| CommandError::new(ErrorCode::WindowFailed, "no main window"))?;
    let capture = app.state::<CaptureWindow>();
    let mut compact = capture
        .0
        .lock()
        .map_err(CommandError::poisoned("capture window"))?;
    if compact.before.is_none() || !pill_measure_ok(width, height) {
        return Ok(());
    }
    apply_pill_bounds(&window, width, height, !compact.placed)?;
    compact.placed = true;
    Ok(())
}

fn apply_pill_bounds(
    window: &WebviewWindow,
    width: f64,
    height: f64,
    move_origin: bool,
) -> Result<(), CommandError> {
    if !pill_measure_ok(width, height) {
        return Ok(());
    }
    window
        .set_size(LogicalSize::new(width, height))
        .map_err(CommandError::with(ErrorCode::WindowFailed))?;
    if move_origin {
        position_pill(window, width, height)?;
    }
    Ok(())
}

fn position_pill(window: &WebviewWindow, width: f64, height: f64) -> Result<(), CommandError> {
    let Some(monitor) = window
        .current_monitor()
        .ok()
        .flatten()
        .or_else(|| window.primary_monitor().ok().flatten())
    else {
        return Err(CommandError::new(ErrorCode::WindowFailed, "no monitor"));
    };
    let scale = monitor.scale_factor();
    let area = monitor.work_area();
    let pill = LogicalSize::new(width, height).to_physical::<i32>(scale);
    // Windows keeps an invisible resize border around the pill; the
    // position applies to that outer edge, the pill is the inner area.
    let (outer, inner) = (
        window
            .outer_position()
            .map_err(CommandError::with(ErrorCode::WindowFailed))?,
        window
            .inner_position()
            .map_err(CommandError::with(ErrorCode::WindowFailed))?,
    );
    let area_width = i32::try_from(area.size.width).unwrap_or(i32::MAX);
    let (x, y) = pill_origin(
        area.position.x,
        area.position.y,
        area_width,
        pill.width,
        scale,
        inner.x - outer.x,
        inner.y - outer.y,
    );
    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(CommandError::with(ErrorCode::WindowFailed))?;
    Ok(())
}

fn default_geometry(window: &WebviewWindow) -> Geometry {
    let Some(monitor) = window
        .primary_monitor()
        .ok()
        .flatten()
        .or_else(|| window.current_monitor().ok().flatten())
    else {
        return Geometry {
            position: PhysicalPosition::new(80, 80),
            size: PhysicalSize::new(1100, 750),
            maximized: false,
        };
    };
    let scale = monitor.scale_factor();
    let width = physical_len(DEFAULT_WINDOW_WIDTH, scale);
    let height = physical_len(DEFAULT_WINDOW_HEIGHT, scale);
    let area = monitor.work_area();
    let area_width = i32::try_from(area.size.width).unwrap_or(i32::MAX);
    let area_height = i32::try_from(area.size.height).unwrap_or(i32::MAX);
    let window_width = i32::try_from(width).unwrap_or(i32::MAX);
    let window_height = i32::try_from(height).unwrap_or(i32::MAX);
    Geometry {
        position: PhysicalPosition::new(
            area.position.x + (area_width - window_width).max(0) / 2,
            area.position.y + (area_height - window_height).max(0) / 2,
        ),
        size: PhysicalSize::new(width, height),
        maximized: false,
    }
}

fn position_on_screen(position: PhysicalPosition<i32>) -> bool {
    position.x > OFF_SCREEN && position.y > OFF_SCREEN
}

fn restore_is_usable(position: PhysicalPosition<i32>, size: PhysicalSize<u32>) -> bool {
    position_on_screen(position) && size.width > 0 && size.height > 0
}

fn pill_measure_ok(width: f64, height: f64) -> bool {
    width > 0.0 && height > 0.0 && width <= MAX_PILL_WIDTH && height <= MAX_PILL_HEIGHT
}

fn pill_origin(
    area_x: i32,
    area_y: i32,
    area_width: i32,
    pill_width: i32,
    scale: f64,
    frame_x: i32,
    frame_y: i32,
) -> (i32, i32) {
    let x = area_x + (area_width - pill_width).max(0) / 2 - frame_x;
    let y = area_y + physical_len(TOP_MARGIN, scale) as i32 - frame_y;
    (x, y)
}

fn physical_len(logical: f64, scale: f64) -> u32 {
    let px = (logical * scale).round();
    if px.is_finite() && px > 0.0 && px <= f64::from(u32::MAX) {
        // Der Wert liegt in 1..=u32::MAX.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            px as u32
        }
    } else {
        1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capture_window_active_folgt_before() {
        let capture = CaptureWindow::default();
        assert!(!capture.active());
        {
            let mut compact = capture.0.lock().expect("lock");
            compact.before = Some(Geometry {
                position: PhysicalPosition::new(0, 0),
                size: PhysicalSize::new(100, 100),
                maximized: false,
            });
        }
        assert!(capture.active());
        {
            let mut compact = capture.0.lock().expect("lock");
            compact.before = None;
        }
        assert!(!capture.active());
    }

    #[test]
    fn minimierte_parkposition_ist_kein_rueckkehrziel() {
        assert!(!restore_is_usable(
            PhysicalPosition::new(-32_000, -32_000),
            PhysicalSize::new(1100, 750),
        ));
        assert!(!restore_is_usable(
            PhysicalPosition::new(10, 10),
            PhysicalSize::new(0, 0),
        ));
        assert!(restore_is_usable(
            PhysicalPosition::new(10, 10),
            PhysicalSize::new(1100, 750),
        ));
    }

    #[test]
    fn vollfenster_gilt_nicht_als_pill() {
        assert!(pill_measure_ok(420.0, 48.0));
        assert!(!pill_measure_ok(1100.0, 750.0));
        assert!(!pill_measure_ok(0.0, 48.0));
        assert!(!pill_measure_ok(720.0, 400.0));
    }

    #[test]
    fn pill_sitzt_oben_mittig() {
        assert_eq!(pill_origin(0, 0, 1920, 400, 1.0, 0, 0), (760, 12));
        assert_eq!(pill_origin(100, 50, 1000, 200, 2.0, 8, 8), (492, 66));
    }
}
