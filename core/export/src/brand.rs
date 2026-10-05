//! Markenvarianten aus `brands.json`: Farbpaletten und Schrift für Oberfläche
//! und Exporte.
//!
//! Die App liest dieselbe Datei; hier liegt die einzige Quelle der Farbwerte.
//! Die Farbableitung ([`brand_palette`]) rechnet wie `brandPalette` in
//! `app/src/brand.ts`, damit ein Export dieselben Farben wie die App nutzt.

use std::sync::OnceLock;

use serde::Deserialize;

const BRANDS_JSON: &str = include_str!("../brands.json");

/// Mindestkontrast des Akzents zum Hintergrund (WCAG AA für Text).
const ACCENT_CONTRAST: f64 = 4.5;

/// Farben `#RRGGBB` eines Farbschemas. Fehlt in `brands.json` ein Token oder
/// steht dort ein unbekanntes, schlägt schon das Laden mit dem Namen fehl.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Palette {
    /// Hintergrund der Seite.
    pub bg: String,
    /// Leicht abgehobene Fläche.
    pub elev: String,
    /// Karten und Eingabefelder.
    pub raised: String,
    /// Text.
    pub ink: String,
    /// Nebentext.
    pub muted: String,
    /// Linien und Rahmen.
    pub line: String,
    /// Akzent.
    pub accent: String,
    /// Text auf dem Akzent.
    pub on_accent: String,
    /// Gefahr und Fehler.
    pub danger: String,
    /// Filmstreifen im Editor.
    pub film: String,
    /// Text auf dem Filmstreifen.
    pub film_ink: String,
    /// Aufnahmeleiste.
    pub pill: String,
    /// Text auf der Aufnahmeleiste.
    pub pill_ink: String,
    /// Grundfarbe der Schatten.
    pub shadow: String,
}

impl Palette {
    /// Token-Name wie in `brands.json` und Farbe, alphabetisch.
    pub fn tokens(&self) -> [(&'static str, &str); 14] {
        [
            ("accent", self.accent.as_str()),
            ("bg", self.bg.as_str()),
            ("danger", self.danger.as_str()),
            ("elev", self.elev.as_str()),
            ("film", self.film.as_str()),
            ("film_ink", self.film_ink.as_str()),
            ("ink", self.ink.as_str()),
            ("line", self.line.as_str()),
            ("muted", self.muted.as_str()),
            ("on_accent", self.on_accent.as_str()),
            ("pill", self.pill.as_str()),
            ("pill_ink", self.pill_ink.as_str()),
            ("raised", self.raised.as_str()),
            ("shadow", self.shadow.as_str()),
        ]
    }
}

/// Farbschema einer Palette.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skin {
    /// Helles Farbschema, auch für Druck und PDF.
    Light,
    /// Dunkles Farbschema.
    Dark,
}

/// Eine Markenvariante mit heller und dunkler Palette.
#[derive(Debug, Clone, Deserialize)]
pub struct Brand {
    /// Stabiler Code, so steht er in den Einstellungen.
    pub code: String,
    /// Palette für das helle Farbschema.
    pub light: Palette,
    /// Palette für das dunkle Farbschema.
    pub dark: Palette,
}

impl Brand {
    /// Palette zum Farbschema.
    pub fn palette(&self, skin: Skin) -> &Palette {
        match skin {
            Skin::Light => &self.light,
            Skin::Dark => &self.dark,
        }
    }
}

/// Markenschrift mit Ersatzschriften, falls sie nicht geladen werden kann.
#[derive(Debug, Clone, Deserialize)]
pub struct BrandFont {
    /// Familienname, wie ihn die Schriftdatei nennt.
    pub family: String,
    /// Ersatzschriften in Reihenfolge; Kleinbuchstaben-Namen sind generische
    /// CSS-Familien wie `sans-serif`.
    pub fallback: Vec<String>,
}

impl BrandFont {
    /// CSS-`font-family`-Liste: Markenschrift, dann die Ersatzschriften.
    pub fn css_stack(&self) -> String {
        std::iter::once(&self.family)
            .chain(&self.fallback)
            .map(|name| {
                let generic = name.chars().all(|c| c.is_ascii_lowercase() || c == '-');
                if generic {
                    name.clone()
                } else {
                    format!("\"{name}\"")
                }
            })
            .collect::<Vec<_>>()
            .join(",")
    }
}

#[derive(Debug, Deserialize)]
struct Catalog {
    default: String,
    font: BrandFont,
    brands: Vec<Brand>,
}

fn catalog() -> &'static Catalog {
    static CATALOG: OnceLock<Catalog> = OnceLock::new();
    CATALOG.get_or_init(|| {
        serde_json::from_str(BRANDS_JSON).unwrap_or_else(|error| panic!("brands.json: {error}"))
    })
}

/// Alle Markenvarianten in Anzeigereihenfolge.
///
/// # Panics
///
/// Nur wenn die eingebaute `brands.json` ungültig ist; die Tests schließen das aus.
pub fn brands() -> &'static [Brand] {
    &catalog().brands
}

/// Code der Standardvariante.
///
/// # Panics
///
/// Wie [`brands`].
pub fn default_brand() -> &'static str {
    &catalog().default
}

/// Die Markenschrift, gleich für alle Varianten.
///
/// # Panics
///
/// Wie [`brands`].
pub fn font() -> &'static BrandFont {
    &catalog().font
}

/// Variante zu einem Code, `None` bei unbekanntem Code.
pub fn find_brand(code: &str) -> Option<&'static Brand> {
    brands().iter().find(|brand| brand.code == code)
}

/// Variante zu einem Code; unbekannte Codes ergeben die Standardvariante.
///
/// # Panics
///
/// Wie [`brands`].
pub fn brand_or_default(code: &str) -> &'static Brand {
    find_brand(code)
        .or_else(|| find_brand(default_brand()))
        .unwrap_or(&brands()[0])
}

/// Kanäle einer Farbe `#RRGGBB`, `None` bei anderem Format.
fn channels(hex: &str) -> Option<[u8; 3]> {
    let digits = hex.strip_prefix('#')?;
    if digits.len() != 6 || !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let value = u32::from_str_radix(digits, 16).ok()?;
    let [_, r, g, b] = value.to_be_bytes();
    Some([r, g, b])
}

/// RGB-Anteile 0..1 einer Farbe `#RRGGBB`, etwa für PDF-Farboperatoren.
pub fn rgb_unit(hex: &str) -> Option<[f32; 3]> {
    channels(hex).map(|rgb| rgb.map(|c| f32::from(c) / 255.0))
}

fn to_hex(rgb: [f64; 3]) -> String {
    let [r, g, b] = rgb.map(|c| {
        // Nach dem Clamp liegt der Wert in 0..=255.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let byte = c.round().clamp(0.0, 255.0) as u8;
        byte
    });
    format!("#{r:02X}{g:02X}{b:02X}")
}

/// `a` um `amount` (0..1) in Richtung `b` verschoben, in sRGB.
/// Ungültige Farben bleiben unverändert.
pub fn mix_hex(a: &str, b: &str, amount: f64) -> String {
    let (Some(from), Some(to)) = (channels(a), channels(b)) else {
        return a.to_owned();
    };
    let mut mixed = [0.0; 3];
    for (index, channel) in mixed.iter_mut().enumerate() {
        let (x, y) = (f64::from(from[index]), f64::from(to[index]));
        *channel = x + (y - x) * amount;
    }
    to_hex(mixed)
}

fn luminance(hex: &str) -> f64 {
    let [r, g, b] = channels(hex).unwrap_or_default().map(|value| {
        let c = f64::from(value) / 255.0;
        if c <= 0.039_28 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    });
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Kontrastverhältnis nach WCAG, 1 bis 21.
pub fn contrast(a: &str, b: &str) -> f64 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

/// Eigener Akzent, in Richtung Textfarbe verschoben, bis er auf dem
/// Hintergrund lesbar ist.
pub fn readable_accent(accent: &str, palette: &Palette) -> String {
    let (ink, bg) = (&palette.ink, &palette.bg);
    for step in 0..=10 {
        let candidate = mix_hex(accent, ink, f64::from(step) / 10.0);
        if contrast(&candidate, bg) >= ACCENT_CONTRAST {
            return candidate;
        }
    }
    ink.clone()
}

/// Palette einer Variante und eines Farbschemas, optional mit eigenem Akzent
/// (`#RRGGBB`; leer oder ungültig = Akzent der Variante).
pub fn brand_palette(code: &str, skin: Skin, custom: &str) -> Palette {
    let mut palette = brand_or_default(code).palette(skin).clone();
    if channels(custom).is_none() {
        return palette;
    }
    let accent = readable_accent(custom, &palette);
    let on_accent = [&palette.ink, &palette.bg]
        .into_iter()
        .fold(&palette.on_accent, |best, candidate| {
            if contrast(candidate, &accent) > contrast(best, &accent) {
                candidate
            } else {
                best
            }
        })
        .clone();
    palette.accent = accent;
    palette.on_accent = on_accent;
    palette
}

#[cfg(test)]
mod tests {
    use super::*;

    fn palettes() -> impl Iterator<Item = (String, &'static Palette)> {
        brands().iter().flat_map(|brand| {
            [
                (format!("{} light", brand.code), &brand.light),
                (format!("{} dark", brand.code), &brand.dark),
            ]
        })
    }

    #[test]
    fn default_exists_and_codes_are_unique() {
        assert!(find_brand(default_brand()).is_some());
        let mut codes: Vec<&str> = brands().iter().map(|b| b.code.as_str()).collect();
        let count = codes.len();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), count);
        assert!(
            (2..=3).contains(&count),
            "two or three presets, got {count}"
        );
    }

    fn token<'a>(palette: &'a Palette, name: &str) -> &'a str {
        palette
            .tokens()
            .into_iter()
            .find_map(|(token, value)| (token == name).then_some(value))
            .unwrap_or_else(|| panic!("unknown token {name}"))
    }

    #[test]
    fn every_brand_in_the_json_has_exactly_the_palette_tokens() {
        let raw: serde_json::Value = serde_json::from_str(BRANDS_JSON).expect("brands.json");
        let tokens: Vec<&str> = brands()[0].light.tokens().map(|(name, _)| name).to_vec();
        let entries = raw["brands"].as_array().expect("brands");
        assert_eq!(entries.len(), brands().len());
        for entry in entries {
            for skin in ["light", "dark"] {
                let mut keys: Vec<&str> = entry[skin]
                    .as_object()
                    .unwrap_or_else(|| panic!("{} {skin} missing", entry["code"]))
                    .keys()
                    .map(String::as_str)
                    .collect();
                keys.sort_unstable();
                assert_eq!(keys, tokens, "{} {skin}", entry["code"]);
            }
        }
    }

    #[test]
    fn palette_tokens_are_hex_colors() {
        for (name, palette) in palettes() {
            for (token, value) in palette.tokens() {
                assert!(channels(value).is_some(), "{name} {token} = {value}");
            }
        }
    }

    #[test]
    fn a_missing_token_fails_with_its_name() {
        let mut light: serde_json::Map<String, serde_json::Value> = brands()[0]
            .light
            .tokens()
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value.into()))
            .collect();
        light.remove("ink");
        let error = serde_json::from_value::<Palette>(serde_json::Value::Object(light))
            .expect_err("ink fehlt");
        assert!(error.to_string().contains("`ink`"), "{error}");
    }

    #[test]
    fn text_colors_meet_wcag_aa() {
        for (name, p) in palettes() {
            for (fg, bg, min) in [
                ("ink", "bg", 7.0),
                ("ink", "elev", 7.0),
                ("ink", "raised", 7.0),
                ("muted", "bg", 4.5),
                ("muted", "elev", 4.5),
                ("muted", "raised", 4.5),
                ("accent", "bg", 4.5),
                ("accent", "elev", 4.5),
                ("on_accent", "accent", 4.5),
                ("danger", "bg", 4.5),
                ("film_ink", "film", 7.0),
                ("pill_ink", "pill", 7.0),
            ] {
                let ratio = contrast(token(p, fg), token(p, bg));
                assert!(
                    ratio >= min,
                    "{name}: {fg} on {bg} is {ratio:.2}, needs {min}"
                );
            }
        }
    }

    #[test]
    fn mix_and_hex_round_like_the_app() {
        assert_eq!(mix_hex("#000000", "#FFFFFF", 0.5), "#808080");
        assert_eq!(mix_hex("#123456", "#FFFFFF", 0.0), "#123456");
        assert_eq!(mix_hex("teal", "#FFFFFF", 0.5), "teal");
        assert_eq!(rgb_unit("#FF0000"), Some([1.0, 0.0, 0.0]));
        assert_eq!(rgb_unit("#12345"), None);
    }

    #[test]
    fn unknown_brand_and_invalid_accent_keep_the_preset() {
        let default = brand_or_default(default_brand());
        assert_eq!(brand_or_default("neon").code, default.code);
        for own in ["", "teal", "#12345"] {
            assert_eq!(&brand_palette("neon", Skin::Light, own), &default.light);
        }
        assert_eq!(
            &brand_palette(default_brand(), Skin::Dark, ""),
            &default.dark
        );
    }

    #[test]
    fn own_accent_is_made_readable_in_both_themes() {
        for brand in brands() {
            for skin in [Skin::Light, Skin::Dark] {
                for own in [
                    "#FFE14D", "#1E40AF", "#10B981", "#000000", "#FFFFFF", "#E11D48",
                ] {
                    let palette = brand_palette(&brand.code, skin, own);
                    let label = format!("{} {skin:?} {own}", brand.code);
                    assert!(
                        contrast(&palette.accent, &palette.bg) >= ACCENT_CONTRAST,
                        "{label}: accent on bg"
                    );
                    assert!(
                        contrast(&palette.on_accent, &palette.accent) >= 3.0,
                        "{label}: text on accent"
                    );
                }
            }
        }
    }

    /// Dieselben Werte prüft `brand.test.ts`; beide Seiten rechnen gleich.
    #[test]
    fn derivation_matches_the_app() {
        for (code, skin, own, accent, on_accent) in PARITY {
            let palette = brand_palette(code, *skin, own);
            assert_eq!(
                (palette.accent.as_str(), palette.on_accent.as_str()),
                (*accent, *on_accent),
                "{code} {skin:?} {own}"
            );
        }
    }

    const PARITY: &[(&str, Skin, &str, &str, &str)] = &[
        ("sage", Skin::Light, "#FFE14D", "#5F5D2E", "#FFFFFF"),
        ("sage", Skin::Dark, "#1E40AF", "#6F86C6", "#0E1512"),
        ("ink", Skin::Light, "#AA3366", "#AA3366", "#FFFFFF"),
        ("ember", Skin::Dark, "#10B981", "#10B981", "#1A1512"),
    ];

    #[test]
    fn font_stack_quotes_names_but_not_generic_families() {
        let font = font();
        assert_ne!(font.family, "");
        let stack = font.css_stack();
        assert!(
            stack.starts_with(&format!("\"{}\"", font.family)),
            "{stack}"
        );
        assert!(stack.ends_with("sans-serif"), "{stack}");
        assert!(!stack.contains("\"sans-serif\""), "{stack}");
    }
}
