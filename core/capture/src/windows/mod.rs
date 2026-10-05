//! Windows-Implementierungen der Capture-Traits (Spec §5).
//!
//! - [`WindowsInputListener`]: globale Low-Level-Hooks via `SetWindowsHookEx`
//!   (`WH_MOUSE_LL`, `WH_KEYBOARD_LL`).
//! - [`WindowsScreenCapturer`]: Screenshots des Klick-Monitors via `xcap`,
//!   physische Pixel + Skalierungsfaktor (Per-Monitor-DPI).
//! - [`WindowsElementInspector`]: UIA-Hit-Test via `uiautomation`-Crate,
//!   Fenstertitel/App-Name über direkte Win32-FFI (`windows`-Crate).
//!
//! Dieser Code wird durch die CI auf `windows-latest` kompiliert und
//! getestet; auf anderen Plattformen ist das Modul per
//! `#[cfg(target_os = "windows")]` ausgeblendet.

mod input;
mod screen;
mod uia;

pub use input::{system_double_click_threshold, WindowsInputListener};
pub use screen::WindowsScreenCapturer;
pub use uia::WindowsElementInspector;

pub(crate) use crate::now_ms;
