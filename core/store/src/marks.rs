//! Standardwerte der Markierungen aus `marks.json`.
//!
//! Die App (`app/src/annotate.ts`) liest dieselbe Datei, der Export zeichnet
//! mit denselben Werten. Jeder Wert hat in `why` eine Begründung.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Deserialize;

const MARKS_JSON: &str = include_str!("../marks.json");

/// Maße der Pfeilspitze in Bildpixeln.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArrowHead {
    /// Länge der Spitze je Pixel Linienbreite.
    pub per_stroke: f64,
    /// Kleinste Länge.
    pub min: f64,
    /// Größte Länge.
    pub max: f64,
    /// Halbe Breite der Spitze relativ zu ihrer Länge.
    pub width: f64,
}

impl ArrowHead {
    /// Länge und halbe Breite der Spitze zu einer Linienbreite.
    pub fn size(&self, stroke: f64) -> (f64, f64) {
        let length = (stroke * self.per_stroke).clamp(self.min, self.max);
        (length, length * self.width)
    }
}

/// Geschätzte Fläche einer Beschriftung relativ zur Textgröße.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextBox {
    /// Breite eines Zeichens je Pixel Textgröße.
    pub char_width: f64,
    /// Zeilenhöhe je Pixel Textgröße.
    pub line_height: f64,
}

/// Textgröße beim Ziehen am Ziehpunkt einer Beschriftung.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextResize {
    /// Gezogene Breite (Anteil am Bild), bei der die Standardgröße gilt.
    pub drag_width: f64,
    /// Kleinste Textgröße in Pixeln.
    pub min: f64,
    /// Größte Textgröße in Pixeln.
    pub max: f64,
}

/// Heller Rand um exportierte Beschriftungen.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextHalo {
    /// Farbe (`#RRGGBB`).
    pub color: String,
    /// Deckkraft (0 bis 1).
    pub opacity: f64,
}

/// Blockgröße des Weichzeichners im Export.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlurBlock {
    /// Kürzere Seite der Fläche geteilt durch diesen Wert.
    pub per_side: u32,
    /// Kleinster Block in Pixeln.
    pub min: u32,
    /// Größter Block in Pixeln.
    pub max: u32,
}

impl BlurBlock {
    /// Kantenlänge eines Blocks für eine Fläche mit kürzerer Seite `side`.
    pub fn size(&self, side: u32) -> u32 {
        (side / self.per_side.max(1)).clamp(self.min, self.max)
    }
}

/// Standardwerte neuer Markierungen.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Marks {
    /// Standardfarbe (`#RRGGBB`), zugleich Rückfall für ungültige Farben.
    pub color: String,
    /// Weitere Farben der Auswahl im Markieren.
    pub extra_colors: Vec<String>,
    /// Linienbreite von Rechteck, Pfeil und Kreis in Bildpixeln.
    pub stroke: f64,
    /// Stiftbreite relativ zur Linienbreite.
    pub pen_factor: f64,
    /// Farbe des Textmarkers.
    pub highlight_color: String,
    /// Deckkraft des Textmarkers (0 bis 1).
    pub highlight_opacity: f64,
    /// Textgröße in Bildpixeln.
    pub text_size: f64,
    /// Rahmen des Weichzeichners beim Bearbeiten.
    pub blur_outline: String,
    /// Maße der Pfeilspitze.
    pub arrow_head: ArrowHead,
    /// Fläche einer Beschriftung zum Anklicken.
    pub text_box: TextBox,
    /// Textgröße beim Ziehen.
    pub text_resize: TextResize,
    /// Rand exportierter Beschriftungen.
    pub text_halo: TextHalo,
    /// Blockgröße des Weichzeichners.
    pub blur_block: BlurBlock,
    /// Begründung je Wert, Schlüssel wie die Felder.
    pub why: BTreeMap<String, String>,
}

impl Marks {
    /// Stiftbreite zu einer Linienbreite.
    pub fn pen_stroke(&self, stroke: f64) -> f64 {
        stroke * self.pen_factor
    }
}

/// Die eingebauten Standardwerte.
///
/// # Panics
///
/// Nur wenn die eingebaute `marks.json` nicht zum Schema passt; die Tests
/// schließen das aus.
pub fn marks() -> &'static Marks {
    static MARKS: OnceLock<Marks> = OnceLock::new();
    MARKS.get_or_init(|| {
        serde_json::from_str(MARKS_JSON).unwrap_or_else(|error| panic!("marks.json: {error}"))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_hex_color(value: &str) -> bool {
        value.strip_prefix('#').is_some_and(|digits| {
            digits.len() == 6 && digits.chars().all(|c| c.is_ascii_hexdigit())
        })
    }

    #[test]
    fn every_value_has_a_reason() {
        let raw: serde_json::Value = serde_json::from_str(MARKS_JSON).expect("marks.json");
        let mut fields: Vec<&String> = raw
            .as_object()
            .expect("object")
            .keys()
            .filter(|key| *key != "why")
            .collect();
        fields.sort();
        let reasons: Vec<&String> = marks().why.keys().collect();
        assert_eq!(fields, reasons);
        for (field, reason) in &marks().why {
            assert!(!reason.trim().is_empty(), "why.{field} is empty");
        }
    }

    #[test]
    fn colors_are_hex_and_distinct() {
        let marks = marks();
        let mut colors: Vec<&str> = std::iter::once(marks.color.as_str())
            .chain(marks.extra_colors.iter().map(String::as_str))
            .collect();
        for color in colors.iter().copied().chain([
            marks.highlight_color.as_str(),
            marks.blur_outline.as_str(),
            marks.text_halo.color.as_str(),
        ]) {
            assert!(is_hex_color(color), "{color}");
        }
        let count = colors.len();
        colors.sort_unstable();
        colors.dedup();
        assert_eq!(colors.len(), count, "a color appears twice");
    }

    #[test]
    fn numbers_are_in_range() {
        let marks = marks();
        assert!(marks.stroke > 0.0 && marks.pen_factor > 0.0 && marks.text_size > 0.0);
        assert!((0.0..=1.0).contains(&marks.highlight_opacity));
        let head = marks.arrow_head;
        assert!(0.0 < head.min && head.min <= head.max && head.width > 0.0);
        assert!(marks.text_box.char_width > 0.0 && marks.text_box.line_height > 0.0);
        let resize = marks.text_resize;
        assert!(resize.drag_width > 0.0 && 0.0 < resize.min && resize.min <= marks.text_size);
        assert!(marks.text_size <= resize.max);
        assert!((0.0..=1.0).contains(&marks.text_halo.opacity));
        let blur = marks.blur_block;
        assert!(blur.per_side > 0 && 0 < blur.min && blur.min <= blur.max);
    }

    #[test]
    fn text_and_blur_values_stay_as_before() {
        let marks = marks();
        assert_eq!(
            marks.text_box,
            TextBox {
                char_width: 0.55,
                line_height: 1.2
            }
        );
        assert_eq!(
            marks.text_resize,
            TextResize {
                drag_width: 0.12,
                min: 10.0,
                max: 48.0
            }
        );
        assert_eq!(marks.text_halo.color, "#FFFFFF");
        assert!((marks.text_halo.opacity - 0.7).abs() < 1e-9);
        assert_eq!(marks.blur_block.size(30), 8);
        assert_eq!(marks.blur_block.size(96), 16);
        assert_eq!(marks.blur_block.size(600), 24);
    }

    #[test]
    fn default_pen_and_arrow_stay_as_before() {
        let marks = marks();
        assert!((marks.pen_stroke(marks.stroke) - 3.0).abs() < 1e-9);
        assert_eq!(marks.arrow_head.size(1.0), (8.0, 8.0 * 0.55));
        assert_eq!(marks.arrow_head.size(marks.stroke), (10.0, 10.0 * 0.55));
        assert_eq!(marks.arrow_head.size(10.0), (18.0, 18.0 * 0.55));
    }
}
