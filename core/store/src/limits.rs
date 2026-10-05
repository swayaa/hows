//! Grenzen für `.steps`-Dateien, die von außen kommen. Jeder Leseweg prüft
//! den Guide hier, bevor App, CLI oder Export ihn sehen; eine Datei, die eine
//! Grenze überschreitet, gilt als beschädigt und wird nicht teilweise geöffnet.

use std::collections::HashSet;

use crate::model::{Action, Crop, Guide, Overlay, Step};

/// Höchstzahl der Einträge im ZIP-Container.
pub const MAX_ZIP_ENTRIES: usize = 10_000;
/// Höchstgröße von `guide.json` nach dem Entpacken, in Bytes (16 MiB).
pub const MAX_GUIDE_JSON_BYTES: u64 = 16_777_216;
/// Höchstgröße eines Screenshots, in Bytes (32 MiB).
pub const MAX_IMAGE_BYTES: u64 = 33_554_432;
/// Höchstgröße aller geladenen Screenshots zusammen, in Bytes (1 GiB).
pub const MAX_TOTAL_IMAGE_BYTES: u64 = 1_073_741_824;
/// Längste Kante eines Screenshots laut PNG-Kopf, in Pixeln.
pub const MAX_IMAGE_SIDE: u32 = 16_384;
/// Höchste Pixelzahl eines Screenshots laut PNG-Kopf.
pub const MAX_IMAGE_PIXELS: u64 = 50_000_000;
/// Höchstzahl der Schritte eines Guides.
pub const MAX_STEPS: usize = 5_000;
/// Längste Schritt-ID, in Zeichen.
pub const MAX_STEP_ID_CHARS: usize = 64;
/// Höchstzahl der Markierungen eines Schritts.
pub const MAX_OVERLAYS_PER_STEP: usize = 500;
/// Höchstzahl der Punkte eines Stiftstrichs.
pub const MAX_PEN_POINTS: usize = 10_000;
/// Längster Freitext (Beschreibung, Schritt-Text, Elementname, Beschriftung), in Zeichen.
pub const MAX_TEXT_CHARS: usize = 10_000;
/// Längste übrige Zeichenkette (Titel, App, Fenster, Farbe, IDs), in Zeichen.
pub const MAX_LABEL_CHARS: usize = 2_000;
/// Kleinster Wert einer normalisierten Markierungs-Koordinate oder -Größe.
pub const MIN_OVERLAY_COORD: f64 = -2.0;
/// Größter Wert einer normalisierten Markierungs-Koordinate oder -Größe.
pub const MAX_OVERLAY_COORD: f64 = 3.0;
/// Größte Strichstärke einer Markierung, in Bildpixeln.
pub const MAX_OVERLAY_STROKE: f64 = 32.0;
/// Größte Schriftgröße einer Beschriftung, in Bildpixeln.
pub const MAX_OVERLAY_TEXT_SIZE: f64 = 64.0;

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1A, b'\n'];

type Check = Result<(), String>;

/// Schritt-IDs sind zugleich Dateinamen (`images/{id}.png`, Markdown-Export),
/// deshalb nur ASCII-Buchstaben, Ziffern, `-` und `_`, ohne Windows-Gerätenamen.
pub(crate) fn is_valid_step_id(id: &str) -> bool {
    (1..=MAX_STEP_ID_CHARS).contains(&id.len())
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
        && !is_windows_device_name(id)
}

fn is_windows_device_name(id: &str) -> bool {
    let upper = id.to_ascii_uppercase();
    match upper.as_bytes() {
        b"CON" | b"PRN" | b"AUX" | b"NUL" => true,
        [b'C', b'O', b'M', digit] | [b'L', b'P', b'T', digit] => digit.is_ascii_digit(),
        _ => false,
    }
}

/// Prüft einen frisch gelesenen Guide gegen alle Grenzen.
pub(crate) fn check_guide(guide: &Guide) -> Check {
    at_most(guide.steps.len(), MAX_STEPS, "steps")?;
    label(&guide.title, "title")?;
    label(&guide.language, "language")?;
    if let Some(description) = &guide.description {
        text(description, "description")?;
    }
    if let Some(environment) = &guide.environment {
        for value in [&environment.os, &environment.os_version]
            .into_iter()
            .flatten()
        {
            label(value, "environment")?;
        }
    }
    let mut ids = HashSet::new();
    for (index, step) in guide.steps.iter().enumerate() {
        let number = index + 1;
        check_step(step).map_err(|reason| format!("step {number}: {reason}"))?;
        if !ids.insert(step.id.to_ascii_lowercase()) {
            return Err(format!("step {number}: duplicate id"));
        }
    }
    Ok(())
}

fn check_step(step: &Step) -> Check {
    if !is_valid_step_id(&step.id) {
        return Err("id is not allowed".to_owned());
    }
    if let Action::KeyCombo { combo } = &step.action {
        label(combo, "combo")?;
    }
    if let Some(element) = &step.element {
        if let Some(name) = &element.name {
            text(name, "element name")?;
        }
        for value in [
            &element.control_type,
            &element.automation_id,
            &element.window_title,
            &element.app_name,
            &element.selector,
            &element.aria_label,
            &element.url,
            &element.page_title,
        ]
        .into_iter()
        .flatten()
        {
            label(value, "element")?;
        }
    }
    text(&step.text.generated, "text")?;
    if let Some(override_text) = &step.text.override_text {
        text(override_text, "text")?;
    }
    at_most(step.overlays.len(), MAX_OVERLAYS_PER_STEP, "overlays")?;
    step.overlays.iter().try_for_each(check_overlay)?;
    step.crop.as_ref().map(check_crop).transpose()?;
    Ok(())
}

fn check_crop(crop: &Crop) -> Check {
    within(crop.x, 0.0, 1.0, "crop x")?;
    within(crop.y, 0.0, 1.0, "crop y")?;
    within(crop.w, 0.0, 1.0, "crop w")?;
    within(crop.h, 0.0, 1.0, "crop h")?;
    if crop.w <= 0.0 || crop.h <= 0.0 {
        return Err("crop is empty".to_owned());
    }
    if crop.x + crop.w > 1.0 + 1e-4 || crop.y + crop.h > 1.0 + 1e-4 {
        return Err("crop leaves the image".to_owned());
    }
    Ok(())
}

fn check_overlay(overlay: &Overlay) -> Check {
    let (id, color, stroke, coords) = match overlay {
        Overlay::Rect {
            id,
            color,
            stroke,
            x,
            y,
            w,
            h,
        } => (id, Some(color), Some(*stroke), vec![*x, *y, *w, *h]),
        Overlay::Arrow {
            id,
            color,
            stroke,
            x1,
            y1,
            x2,
            y2,
        } => (id, Some(color), Some(*stroke), vec![*x1, *y1, *x2, *y2]),
        Overlay::Pen {
            id,
            color,
            stroke,
            points,
        } => {
            at_most(points.len(), MAX_PEN_POINTS, "pen points")?;
            (
                id,
                Some(color),
                Some(*stroke),
                points.as_flattened().to_vec(),
            )
        }
        Overlay::Highlight {
            id,
            color,
            opacity,
            x,
            y,
            w,
            h,
        } => {
            within(*opacity, 0.0, 1.0, "opacity")?;
            (id, Some(color), None, vec![*x, *y, *w, *h])
        }
        Overlay::Text {
            id,
            color,
            size,
            x,
            y,
            text: label_text,
        } => {
            within(*size, 0.0, MAX_OVERLAY_TEXT_SIZE, "text size")?;
            text(label_text, "text overlay")?;
            (id, Some(color), None, vec![*x, *y])
        }
        Overlay::Circle {
            id,
            color,
            stroke,
            cx,
            cy,
            r,
        } => (id, Some(color), Some(*stroke), vec![*cx, *cy, *r]),
        Overlay::Blur { id, x, y, w, h } => (id, None, None, vec![*x, *y, *w, *h]),
    };
    label(id, "overlay id")?;
    if let Some(color) = color {
        label(color, "overlay color")?;
    }
    if let Some(stroke) = stroke {
        within(stroke, 0.0, MAX_OVERLAY_STROKE, "stroke")?;
    }
    coords.into_iter().try_for_each(|value| {
        within(
            value,
            MIN_OVERLAY_COORD,
            MAX_OVERLAY_COORD,
            "overlay coordinate",
        )
    })
}

/// Lehnt zu große Bilder ab und PNGs, deren Kopf eine zu große Fläche
/// ankündigt. Bytes ohne PNG-Kopf dekodiert kein Rust-Pfad: `image` ist nur
/// mit PNG gebaut, und PNG verlangt `IHDR` als ersten Block.
pub(crate) fn check_image(png: &[u8]) -> Check {
    if png.len() as u64 > MAX_IMAGE_BYTES {
        return Err(format!("image exceeds {MAX_IMAGE_BYTES} bytes"));
    }
    let Some(header) = png.get(..24) else {
        return Ok(());
    };
    if header[..8] != PNG_SIGNATURE || &header[12..16] != b"IHDR" {
        return Ok(());
    }
    let width = u32::from_be_bytes([header[16], header[17], header[18], header[19]]);
    let height = u32::from_be_bytes([header[20], header[21], header[22], header[23]]);
    if width > MAX_IMAGE_SIDE
        || height > MAX_IMAGE_SIDE
        || u64::from(width) * u64::from(height) > MAX_IMAGE_PIXELS
    {
        return Err(format!("image of {width}x{height} pixels is too large"));
    }
    Ok(())
}

/// Zählt ein Bild zur Summe aller Bilder eines Containers.
pub(crate) fn add_image_bytes(total: &mut u64, len: usize) -> Check {
    *total += len as u64;
    if *total > MAX_TOTAL_IMAGE_BYTES {
        return Err(format!("images exceed {MAX_TOTAL_IMAGE_BYTES} bytes"));
    }
    Ok(())
}

fn at_most(count: usize, max: usize, what: &str) -> Check {
    if count > max {
        return Err(format!("more than {max} {what}"));
    }
    Ok(())
}

fn label(value: &str, what: &str) -> Check {
    chars(value, MAX_LABEL_CHARS, what)
}

fn text(value: &str, what: &str) -> Check {
    chars(value, MAX_TEXT_CHARS, what)
}

fn chars(value: &str, max: usize, what: &str) -> Check {
    if value.chars().count() > max {
        return Err(format!("{what} is longer than {max} characters"));
    }
    Ok(())
}

fn within(value: f64, min: f64, max: f64, what: &str) -> Check {
    if !(min..=max).contains(&value) {
        return Err(format!("{what} {value} is outside {min} to {max}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ganzes_bild_ist_kein_zuschnitt() {
        let full = Crop {
            x: 0.0,
            y: 0.0,
            w: 1.0,
            h: 1.0,
        };
        assert_eq!(full.normalized(), Ok(None));
        let window = Crop {
            x: 0.1,
            y: 0.2,
            w: 0.3,
            h: 0.4,
        };
        assert_eq!(window.normalized(), Ok(Some(window)));
        assert!(Crop {
            x: f64::NAN,
            y: 0.0,
            w: 0.5,
            h: 0.5,
        }
        .normalized()
        .is_err());
    }

    #[test]
    fn bildsumme_lehnt_mehr_als_die_grenze_ab() {
        let mut total = MAX_TOTAL_IMAGE_BYTES;
        assert!(add_image_bytes(&mut total, 1).is_err());
    }
}
