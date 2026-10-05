//! Element-Inspektion via UI Automation (`uiautomation`-Crate) mit
//! Win32-FFI-Ergänzungen für Fenstertitel und App-Namen (Spec §5).

use std::ffi::c_void;
use std::ptr::null_mut;

use uiautomation::types::Point as UiaPoint;
use uiautomation::{UIAutomation, UIElement};
use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{CloseHandle, HWND, MAX_PATH, POINT};
use windows::Win32::Storage::FileSystem::{
    GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
};
use windows::Win32::System::Threading::{
    OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
};
use windows::Win32::UI::WindowsAndMessaging::{
    GetAncestor, GetForegroundWindow, GetWindowTextW, WindowFromPoint, GA_ROOT,
};

use crate::{CaptureError, CapturedElement, ElementInspector, Point};

/// [`ElementInspector`] für Windows: UIA-Hit-Test plus Win32-Fallbacks.
///
/// Muss auf dem Thread erzeugt werden, auf dem er verwendet wird
/// (COM-Initialisierung durch das `uiautomation`-Crate).
pub struct WindowsElementInspector {
    automation: UIAutomation,
}

impl WindowsElementInspector {
    /// Initialisiert UI Automation (inklusive COM) auf dem aktuellen Thread.
    pub fn new() -> Result<Self, CaptureError> {
        let automation =
            UIAutomation::new().map_err(|error| CaptureError::Inspection(error.to_string()))?;
        Ok(Self { automation })
    }
}

fn non_empty(value: String) -> Option<String> {
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

/// Fenstertitel des Top-Level-Fensters eines Handles.
pub(crate) fn window_title(hwnd: HWND) -> Option<String> {
    if hwnd.is_invalid() {
        return None;
    }
    // SAFETY: Gültiges HWND; GetWindowTextW schreibt maximal `len - 1`
    // u16-Zeichen plus Nullterminator in den Puffer.
    unsafe {
        let root = GetAncestor(hwnd, GA_ROOT);
        let target = if root.is_invalid() { hwnd } else { root };
        let mut buffer = [0u16; 512];
        let length = GetWindowTextW(target, &mut buffer);
        if length <= 0 {
            return None;
        }
        let length = usize::try_from(length).ok()?;
        non_empty(String::from_utf16_lossy(&buffer[..length]))
    }
}

/// Pfad des Prozess-Images zu einer Prozess-ID.
fn image_path_for_pid(pid: u32) -> Option<String> {
    if pid == 0 {
        return None;
    }
    // SAFETY: Handle wird geprüft und nach Gebrauch geschlossen; der Puffer
    // ist groß genug für MAX_PATH und `size` beschreibt seine Kapazität.
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buffer = [0u16; MAX_PATH as usize];
        let mut size = u32::try_from(buffer.len()).ok()?;
        let result = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(buffer.as_mut_ptr()),
            &raw mut size,
        );
        CloseHandle(handle).ok();
        result.ok()?;
        Some(String::from_utf16_lossy(&buffer[..size as usize]))
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}

/// Bytes eines Eintrags aus einem Versionsinfo-Block. Windows meldet die Länge
/// in Einheiten (Bytes bei binären, UTF-16-Zeichen bei Text-Einträgen), daher
/// `unit`. Liegt der Bereich nicht vollständig im Block, gibt es `None`.
fn version_value<'a>(block: &'a [u8], sub_block: &str, unit: usize) -> Option<&'a [u8]> {
    let sub_block = wide(sub_block);
    let mut value: *mut c_void = null_mut();
    let mut length = 0u32;
    // SAFETY: `block` stammt von GetFileVersionInfoW und lebt während des
    // Aufrufs; alle Zeiger zeigen auf gültigen Speicher.
    let found = unsafe {
        VerQueryValueW(
            block.as_ptr().cast(),
            PCWSTR(sub_block.as_ptr()),
            &raw mut value,
            &raw mut length,
        )
    };
    if !found.as_bool() || value.is_null() {
        return None;
    }
    let start = (value as usize).checked_sub(block.as_ptr() as usize)?;
    let end = start.checked_add((length as usize).checked_mul(unit)?)?;
    block.get(start..end).filter(|bytes| !bytes.is_empty())
}

/// Produktbeschreibung aus den Versionsinfos der EXE, etwa
/// `Windows PowerShell` für `powershell.exe`.
fn file_description(path: &str) -> Option<String> {
    let file = wide(path);
    // SAFETY: `file` ist nullterminiert; der Puffer hat die zuvor
    // abgefragte Größe.
    let block = unsafe {
        let size = GetFileVersionInfoSizeW(PCWSTR(file.as_ptr()), None);
        if size == 0 {
            return None;
        }
        let mut block = vec![0u8; size as usize];
        GetFileVersionInfoW(PCWSTR(file.as_ptr()), None, size, block.as_mut_ptr().cast()).ok()?;
        block
    };
    // Erste Sprach-/Codepage-Kombination, sonst US-Englisch/Unicode.
    let (language, codepage) = match version_value(&block, "\\VarFileInfo\\Translation", 1) {
        Some([l0, l1, c0, c1, ..]) => (
            u16::from_le_bytes([*l0, *l1]),
            u16::from_le_bytes([*c0, *c1]),
        ),
        _ => (0x0409, 0x04B0),
    };
    let key = format!("\\StringFileInfo\\{language:04x}{codepage:04x}\\FileDescription");
    let units: Vec<u16> = version_value(&block, &key, 2)?
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes(*pair))
        .collect();
    let text = String::from_utf16_lossy(&units);
    non_empty(text.trim_end_matches('\0').trim().to_owned())
}

/// Lesbarer App-Name zu einer Prozess-ID: die Produktbeschreibung der EXE,
/// sonst ihr Dateiname ohne Endung.
fn app_name_for_pid(pid: u32) -> Option<String> {
    let path = image_path_for_pid(pid)?;
    file_description(&path).or_else(|| {
        let file_name = path.rsplit(['\\', '/']).next()?;
        let app = file_name.strip_suffix(".exe").unwrap_or(file_name);
        non_empty(app.to_owned())
    })
}

/// Wandelt ein UIA-Element in [`CapturedElement`] um. Jede Teilabfrage darf
/// scheitern (Spec §12).
fn convert(element: &UIElement, window: Option<HWND>) -> CapturedElement {
    let window_title = window.and_then(window_title);
    let app_name = element.get_process_id().ok().and_then(app_name_for_pid);
    CapturedElement {
        name: element.get_name().ok().and_then(non_empty),
        control_type: element
            .get_control_type()
            .ok()
            .map(|control_type| format!("{control_type:?}")),
        automation_id: element.get_automation_id().ok().and_then(non_empty),
        window_title,
        app_name,
    }
}

impl ElementInspector for WindowsElementInspector {
    fn element_at(&mut self, position: Point) -> Result<CapturedElement, CaptureError> {
        // SAFETY: Reiner Koordinaten-Hit-Test ohne Zeiger-Übergabe.
        let hwnd = unsafe {
            WindowFromPoint(POINT {
                x: position.x,
                y: position.y,
            })
        };

        match self
            .automation
            .element_from_point(UiaPoint::new(position.x, position.y))
        {
            Ok(element) => Ok(convert(&element, Some(hwnd))),
            // UIA liefert bei manchen Apps nichts (Spec §12): Schritt ohne
            // Element-Namen, aber mit Fenstertitel/App-Name, wo möglich.
            Err(_) => Ok(CapturedElement {
                window_title: window_title(hwnd),
                ..CapturedElement::default()
            }),
        }
    }

    fn focused_element(&mut self) -> Result<CapturedElement, CaptureError> {
        // SAFETY: Parameterloser Win32-Aufruf.
        let hwnd = unsafe { GetForegroundWindow() };
        match self.automation.get_focused_element() {
            Ok(element) => Ok(convert(&element, Some(hwnd))),
            Err(_) => Ok(CapturedElement {
                window_title: window_title(hwnd),
                ..CapturedElement::default()
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Läuft auf der Windows-CI: Inspektion darf nie hart scheitern,
    /// sondern degradiert zu leeren Feldern.
    #[test]
    fn element_at_degradiert_graceful() {
        let Ok(mut inspector) = WindowsElementInspector::new() else {
            eprintln!("UIA nicht verfügbar (Headless-Runner?)");
            return;
        };
        let element = inspector
            .element_at(Point { x: 10, y: 10 })
            .expect("element_at darf nicht hart scheitern");
        // Auf dem CI-Desktop existiert immer irgendein Fenster/Element;
        // wir verlangen aber bewusst keine konkreten Inhalte.
        let _ = element;
    }

    #[test]
    fn focused_element_degradiert_graceful() {
        let Ok(mut inspector) = WindowsElementInspector::new() else {
            eprintln!("UIA nicht verfügbar (Headless-Runner?)");
            return;
        };
        inspector
            .focused_element()
            .expect("focused_element darf nicht hart scheitern");
    }

    #[test]
    fn app_name_ist_die_produktbeschreibung() {
        let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_owned());
        // Windows kann die Beschreibung lokalisieren, deshalb kein fester Text.
        let description = file_description(&format!(r"{root}\System32\cmd.exe"))
            .expect("cmd.exe hat eine Produktbeschreibung");
        assert!(description.contains(' '), "{description}");
        assert_eq!(file_description(r"C:\gibt\es\nicht.exe"), None);
    }

    #[test]
    fn app_name_des_eigenen_prozesses() {
        let name = app_name_for_pid(std::process::id()).expect("eigener Prozess");
        assert_ne!(name, "");
        assert_eq!(app_name_for_pid(0), None);
    }
}
