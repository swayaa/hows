//! Screenshots via `xcap`: Monitor am Klickpunkt, physische Pixel,
//! Skalierungsfaktor (Per-Monitor-DPI, Spec §5).

use std::io::Cursor;

use xcap::Monitor;

use crate::{CaptureError, MonitorSnapshot, Point, ScreenCapturer, Screenshot};

/// [`ScreenCapturer`] für Windows auf Basis des `xcap`-Crates.
#[derive(Debug, Default)]
pub struct WindowsScreenCapturer;

fn screenshot_error(error: impl std::fmt::Display) -> CaptureError {
    CaptureError::Screenshot(error.to_string())
}

/// Findet den Monitor, der `position` enthält; Fallback: primärer Monitor,
/// sonst der erste. Liefert zusätzlich den Index in der Systemliste.
fn monitor_at(position: Point) -> Result<(u32, Monitor), CaptureError> {
    let monitors = Monitor::all().map_err(screenshot_error)?;
    if monitors.is_empty() {
        return Err(CaptureError::Screenshot("kein Monitor gefunden".to_owned()));
    }

    let target = Monitor::from_point(position.x, position.y).ok();
    let target_id = target.as_ref().and_then(|monitor| monitor.id().ok());

    // Index über die ID in der Gesamtliste bestimmen (xcap liefert bei
    // `from_point` keinen Index).
    if let Some(target_id) = target_id {
        for (index, monitor) in monitors.iter().enumerate() {
            if monitor.id().ok() == Some(target_id) {
                return Ok((u32::try_from(index).unwrap_or(0), monitor.clone()));
            }
        }
    }

    for (index, monitor) in monitors.iter().enumerate() {
        if monitor.is_primary().unwrap_or(false) {
            return Ok((u32::try_from(index).unwrap_or(0), monitor.clone()));
        }
    }
    Ok((0, monitors[0].clone()))
}

impl ScreenCapturer for WindowsScreenCapturer {
    fn capture_at(&mut self, position: Point) -> Result<Screenshot, CaptureError> {
        let (index, monitor) = monitor_at(position)?;

        let image = monitor.capture_image().map_err(screenshot_error)?;
        let mut png = Vec::new();
        image
            .write_to(&mut Cursor::new(&mut png), image::ImageFormat::Png)
            .map_err(screenshot_error)?;

        Ok(Screenshot {
            png,
            monitor: MonitorSnapshot {
                index,
                x: monitor.x().map_err(screenshot_error)?,
                y: monitor.y().map_err(screenshot_error)?,
                width: monitor.width().map_err(screenshot_error)?,
                height: monitor.height().map_err(screenshot_error)?,
                scale_factor: f64::from(monitor.scale_factor().map_err(screenshot_error)?),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Läuft auf der Windows-CI. Falls der Runner keine Capture-fähige
    /// Sitzung hat, wird der Test übersprungen statt falsch rot zu sein.
    #[test]
    fn screenshot_liefert_png_mit_monitor_metadaten() {
        let mut capturer = WindowsScreenCapturer;
        match capturer.capture_at(Point { x: 0, y: 0 }) {
            Ok(screenshot) => {
                assert!(screenshot.png.len() > 8);
                assert_eq!(&screenshot.png[..8], b"\x89PNG\r\n\x1a\n");
                assert!(screenshot.monitor.width > 0);
                assert!(screenshot.monitor.height > 0);
                assert!(screenshot.monitor.scale_factor > 0.0);
            }
            Err(error) => {
                eprintln!("Screenshot nicht möglich (Headless-Runner?): {error}");
            }
        }
    }
}
