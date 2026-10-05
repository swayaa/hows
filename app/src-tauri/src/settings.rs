//! Lokale App-Einstellungen (`settings.json` im App-Config-Verzeichnis).
//! Rein lokal, kein Netzwerk (Spec §8).

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use steps_export::{brand, ExportStyle, Paper, PdfPage, DEFAULT_MARGIN_MM};
use steps_i18n::{is_system, SYSTEM};
use steps_session::{Locale, Pauses, SCROLL_GAP_MS, TYPING_GAP_MS};
use steps_store::marks;

/// Persistierte Einstellungen.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Globaler Aufnahme-Hotkey (Default `Ctrl+Shift+R`; `Ctrl+Alt+…`
    /// kollidiert auf deutschen Tastaturen mit AltGr).
    pub hotkey: String,
    /// Globaler Pause/Fortsetzen-Hotkey (Default `Ctrl+Shift+P`).
    pub pause_hotkey: String,
    /// UI- und Aufnahme-Sprache: `system` (wie Windows) oder ein Code aus
    /// `steps_i18n::LOCALES`.
    pub language: String,
    /// Zuletzt geöffnete lokale `.steps`-Pfade (MRU).
    pub recent_paths: Vec<String>,
    /// Standard-Exportformat (`html` | `pdf` | `steps`).
    pub export_format: String,
    /// Nach Export Ordner/Datei öffnen.
    pub open_after_export: bool,
    /// Standard-Exportordner (leer = OS Documents).
    pub default_export_folder: String,
    /// Theme (`system` | `light` | `dark`).
    pub theme: String,
    /// Anleitungsordner für `.steps` (leer = Documents/Steps).
    pub guides_folder: String,
    /// Tipp-Pause (ms), nach der eine neue Texteingabe beginnt.
    pub typing_pause_ms: u64,
    /// Scroll-Pause (ms), nach der ein neuer Scroll-Schritt beginnt.
    pub scroll_pause_ms: u64,
    /// Wie viele zuletzt geöffnete Anleitungen die Bibliothek zeigt.
    pub recent_limit: usize,
    /// Titelvorlage mit `{app}`, `{date}` und `{time}`; leer = Vorlage der Sprache.
    pub title_template: String,
    /// Dateinamenvorlage mit `{title}`, `{app}`, `{date}` und `{time}`.
    pub file_name_template: String,
    /// Papierformat des PDF-Exports (`a4` | `letter`).
    pub pdf_paper: String,
    /// Seitenrand des PDF-Exports in Millimetern.
    pub pdf_margin_mm: u32,
    /// Standardfarbe neuer Markierungen (`#RRGGBB`).
    pub annotation_color: String,
    /// Strichstärke neuer Markierungen in Pixeln.
    pub annotation_stroke: f64,
    /// Markenvariante, ein Code aus `brands.json`.
    pub brand: String,
    /// Eigene Akzentfarbe (`#RRGGBB`); leer = Akzent der Markenvariante.
    pub accent_color: String,
    /// „Erstellt mit Hows“ in HTML-, PDF- und Markdown-Exporten.
    pub export_credit: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: "Ctrl+Shift+R".to_owned(),
            pause_hotkey: "Ctrl+Shift+P".to_owned(),
            language: SYSTEM.to_owned(),
            recent_paths: Vec::new(),
            export_format: "html".to_owned(),
            open_after_export: true,
            default_export_folder: String::new(),
            theme: "system".to_owned(),
            guides_folder: String::new(),
            typing_pause_ms: TYPING_GAP_MS,
            scroll_pause_ms: SCROLL_GAP_MS,
            recent_limit: 8,
            title_template: String::new(),
            file_name_template: DEFAULT_FILE_NAME_TEMPLATE.to_owned(),
            pdf_paper: Paper::default().code().to_owned(),
            pdf_margin_mm: DEFAULT_MARGIN_MM,
            annotation_color: marks().color.clone(),
            annotation_stroke: marks().stroke,
            brand: brand::default_brand().to_owned(),
            accent_color: String::new(),
            export_credit: true,
        }
    }
}

/// Erlaubter Bereich eines Zahlenwerts; `step` ist die Schrittweite der UI.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Limit<T> {
    pub min: T,
    pub max: T,
    pub step: T,
}

impl<T: PartialOrd + Copy> Limit<T> {
    fn clamp(self, value: T) -> T {
        if value < self.min {
            self.min
        } else if value > self.max {
            self.max
        } else {
            value
        }
    }
}

/// Grenzen aller Zahlenwerte. Die UI liest sie über `get_settings_defaults`.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Limits {
    pub typing_pause_ms: Limit<u64>,
    pub scroll_pause_ms: Limit<u64>,
    pub recent_limit: Limit<usize>,
    pub pdf_margin_mm: Limit<u32>,
    pub annotation_stroke: Limit<f64>,
}

pub const LIMITS: Limits = Limits {
    typing_pause_ms: Limit {
        min: 500,
        max: 10_000,
        step: 250,
    },
    scroll_pause_ms: Limit {
        min: 250,
        max: 5_000,
        step: 250,
    },
    recent_limit: Limit {
        min: 1,
        max: 30,
        step: 1,
    },
    pdf_margin_mm: Limit {
        min: 5,
        max: 40,
        step: 1,
    },
    annotation_stroke: Limit {
        min: 1.0,
        max: 8.0,
        step: 0.5,
    },
};

/// Eine Änderung aus der UI. Zahlen werden auf [`LIMITS`] begrenzt, Texte
/// normalisiert.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "field", content = "value", rename_all = "snake_case")]
pub enum SettingChange {
    TypingPauseMs(u64),
    ScrollPauseMs(u64),
    RecentLimit(usize),
    TitleTemplate(String),
    FileNameTemplate(String),
    PdfPaper(String),
    PdfMarginMm(u32),
    AnnotationColor(String),
    AnnotationStroke(f64),
    Brand(String),
    AccentColor(String),
    ExportCredit(bool),
}

impl Settings {
    /// Die eingestellte Sprache; `system` folgt Windows, unbekannte Codes
    /// ergeben Englisch.
    pub fn locale(&self) -> Locale {
        Locale::from_setting(&self.language)
    }

    /// Pausen für die Aufnahme.
    pub fn pauses(&self) -> Pauses {
        Pauses {
            typing_ms: self.typing_pause_ms,
            scroll_ms: self.scroll_pause_ms,
        }
    }

    /// Seite des PDF-Exports.
    pub fn pdf_page(&self) -> PdfPage {
        PdfPage {
            paper: Paper::from_code(&self.pdf_paper).unwrap_or_default(),
            margin_mm: self.pdf_margin_mm,
        }
    }

    /// Marke, Akzent und Credit-Zeile der Exporte.
    pub fn export_style(&self) -> ExportStyle {
        ExportStyle {
            brand: self.brand.clone(),
            accent: self.accent_color.clone(),
            credit: self.export_credit,
        }
    }

    /// Eigene Titelvorlage, falls gesetzt.
    pub fn title_template(&self) -> Option<&str> {
        Some(self.title_template.trim()).filter(|template| !template.is_empty())
    }

    /// Übernimmt eine Änderung aus der UI.
    pub fn apply(&mut self, change: SettingChange) {
        match change {
            SettingChange::TypingPauseMs(ms) => {
                self.typing_pause_ms = LIMITS.typing_pause_ms.clamp(ms);
            }
            SettingChange::ScrollPauseMs(ms) => {
                self.scroll_pause_ms = LIMITS.scroll_pause_ms.clamp(ms);
            }
            SettingChange::RecentLimit(limit) => {
                self.recent_limit = LIMITS.recent_limit.clamp(limit);
                self.recent_paths.truncate(self.recent_limit);
            }
            SettingChange::TitleTemplate(template) => {
                self.title_template = template.trim().to_owned();
            }
            SettingChange::FileNameTemplate(template) => {
                self.file_name_template = normalize_file_name_template(&template);
            }
            SettingChange::PdfPaper(code) => {
                self.pdf_paper = Paper::from_code(&code)
                    .unwrap_or_default()
                    .code()
                    .to_owned();
            }
            SettingChange::PdfMarginMm(mm) => self.pdf_margin_mm = LIMITS.pdf_margin_mm.clamp(mm),
            SettingChange::AnnotationColor(color) => {
                if let Some(color) = normalize_color(&color) {
                    self.annotation_color = color;
                }
            }
            SettingChange::AnnotationStroke(px) => {
                if px.is_finite() {
                    self.annotation_stroke = LIMITS.annotation_stroke.clamp(px);
                }
            }
            SettingChange::Brand(code) => {
                self.brand = brand::find_brand(code.trim())
                    .map_or_else(brand::default_brand, |found| found.code.as_str())
                    .to_owned();
            }
            SettingChange::AccentColor(color) => {
                if color.trim().is_empty() {
                    self.accent_color.clear();
                } else if let Some(color) = normalize_color(&color) {
                    self.accent_color = color;
                }
            }
            SettingChange::ExportCredit(on) => self.export_credit = on,
        }
    }

    /// Setzt die Werte unter „Erweitert“ auf ihre Standards zurück.
    pub fn reset_advanced(&mut self) {
        let defaults = Settings::default();
        self.typing_pause_ms = defaults.typing_pause_ms;
        self.scroll_pause_ms = defaults.scroll_pause_ms;
        self.recent_limit = defaults.recent_limit;
        self.recent_paths.truncate(self.recent_limit);
        self.title_template = defaults.title_template;
        self.file_name_template = defaults.file_name_template;
        self.pdf_margin_mm = defaults.pdf_margin_mm;
        self.annotation_stroke = defaults.annotation_stroke;
    }

    /// Bringt eine von Hand bearbeitete Datei in die erlaubten Bereiche.
    fn sanitized(mut self) -> Self {
        let raw = self.clone();
        self.apply(SettingChange::TypingPauseMs(raw.typing_pause_ms));
        self.apply(SettingChange::ScrollPauseMs(raw.scroll_pause_ms));
        self.apply(SettingChange::RecentLimit(raw.recent_limit));
        self.apply(SettingChange::TitleTemplate(raw.title_template));
        self.apply(SettingChange::FileNameTemplate(raw.file_name_template));
        self.apply(SettingChange::PdfPaper(raw.pdf_paper));
        self.apply(SettingChange::PdfMarginMm(raw.pdf_margin_mm));
        self.apply(SettingChange::AnnotationColor(raw.annotation_color));
        self.apply(SettingChange::AnnotationStroke(raw.annotation_stroke));
        if normalize_color(&self.annotation_color).is_none() {
            self.annotation_color.clone_from(&marks().color);
        }
        self.apply(SettingChange::Brand(raw.brand));
        self.apply(SettingChange::AccentColor(raw.accent_color));
        if !self.accent_color.is_empty() && normalize_color(&self.accent_color).is_none() {
            self.accent_color.clear();
        }
        self.repair_shortcuts();
        self
    }

    /// Ein Aufnahme-Kürzel braucht Strg, Alt oder Win. Umschalt allein ist
    /// Tippen, deshalb fällt ein solches Kürzel auf den Standard zurück.
    /// Zwei gleiche Kürzel werden ebenfalls auf die Standards gesetzt.
    fn repair_shortcuts(&mut self) {
        let defaults = Settings::default();
        if !is_recording_shortcut(&self.hotkey) {
            self.hotkey.clone_from(&defaults.hotkey);
        }
        if !is_recording_shortcut(&self.pause_hotkey) {
            self.pause_hotkey.clone_from(&defaults.pause_hotkey);
        }
        if self.hotkey.eq_ignore_ascii_case(&self.pause_hotkey) {
            self.hotkey.clone_from(&defaults.hotkey);
            self.pause_hotkey.clone_from(&defaults.pause_hotkey);
        }
    }
}

/// `#RRGGBB` in Großbuchstaben, sonst `None`.
pub fn normalize_color(raw: &str) -> Option<String> {
    let hex = raw.trim().strip_prefix('#')?;
    (hex.len() == 6 && hex.chars().all(|c| c.is_ascii_hexdigit()))
        .then(|| format!("#{}", hex.to_ascii_uppercase()))
}

/// Vorlage neuer Dateinamen und Ersatz für leere Vorlagen.
pub const DEFAULT_FILE_NAME_TEMPLATE: &str = "{title}";

/// Leere Vorlagen ergeben [`DEFAULT_FILE_NAME_TEMPLATE`]; Pfadtrenner sind nicht erlaubt, damit
/// die Vorlage keine Ordner anlegt.
fn normalize_file_name_template(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| if matches!(c, '/' | '\\') { ' ' } else { c })
        .collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        DEFAULT_FILE_NAME_TEMPLATE.to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn settings_path(config_dir: &Path) -> PathBuf {
    config_dir.join("settings.json")
}

/// Lädt die Einstellungen; fehlende/kaputte Datei ergibt Defaults.
pub fn load(config_dir: &Path) -> Settings {
    std::fs::read(settings_path(config_dir))
        .ok()
        .and_then(|raw| serde_json::from_slice::<Settings>(&raw).ok())
        .unwrap_or_default()
        .sanitized()
}

/// Speichert die Einstellungen (legt das Verzeichnis bei Bedarf an).
/// Die Datei wird erst ersetzt, wenn der neue Inhalt vollständig ist.
pub fn save(config_dir: &Path, settings: &Settings) -> std::io::Result<()> {
    std::fs::create_dir_all(config_dir)?;
    let json = serde_json::to_vec_pretty(settings)
        .map_err(|error| std::io::Error::new(std::io::ErrorKind::InvalidData, error))?;
    let path = settings_path(config_dir);
    steps_store::write_atomically(&path, |writer| {
        writer.write_all(&json)?;
        Ok(())
    })
    .map_err(std::io::Error::other)
}

/// Aufnahme-Steuerung: mindestens Strg, Alt oder Win, dazu eine Taste.
/// Umschalt darf zusätzlich dabei sein. `Shift+F9` und `F9` gelten nicht.
pub fn is_recording_shortcut(chord: &str) -> bool {
    let mut ctrl = false;
    let mut alt = false;
    let mut win = false;
    let mut key = false;
    for part in chord.split('+') {
        let token = part.trim();
        if token.is_empty() {
            return false;
        }
        match token.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => ctrl = true,
            "alt" => alt = true,
            "win" | "meta" | "super" => win = true,
            "shift" => {}
            _ => {
                if key {
                    return false;
                }
                key = true;
            }
        }
    }
    key && (ctrl || alt || win)
}

/// Normalisiert Theme-Werte auf `system` | `light` | `dark`.
pub fn normalize_theme(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "light" | "hell" => "light".to_owned(),
        "dark" | "dunkel" => "dark".to_owned(),
        _ => "system".to_owned(),
    }
}

/// Normalisiert Exportformat auf `html` | `pdf` | `steps`.
pub fn normalize_export_format(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "steps" | ".steps" => "steps".to_owned(),
        "pdf" | ".pdf" => "pdf".to_owned(),
        _ => "html".to_owned(),
    }
}

/// Normalisiert auf `system` oder einen unterstützten Sprachcode, sonst `en`.
pub fn normalize_language(raw: &str) -> String {
    if is_system(raw) {
        SYSTEM.to_owned()
    } else {
        Locale::resolve(raw).code().to_owned()
    }
}

/// Prüft, ob ein Ordnerpfad nutzbar ist (existiert als Dir oder anlegbar).
pub fn folder_is_usable(path: &Path) -> bool {
    if path.as_os_str().is_empty() {
        return false;
    }
    if path.is_dir() {
        return true;
    }
    // Noch nicht vorhanden: Parent muss existieren und anlegbar sein.
    match path.parent() {
        Some(parent) if parent.as_os_str().is_empty() || parent.is_dir() => {
            std::fs::create_dir_all(path).is_ok()
        }
        _ => false,
    }
}

/// Löst den konfigurierten Ordner auf. Ist er leer oder ungültig, gelten der Fallback und das Flag.
pub fn resolve_folder(configured: &str, fallback: PathBuf) -> (PathBuf, bool) {
    let trimmed = configured.trim();
    if trimmed.is_empty() {
        return (fallback, false);
    }
    let path = PathBuf::from(trimmed);
    if folder_is_usable(&path) {
        (path, false)
    } else {
        (fallback, true)
    }
}

/// Liegt der Ordner unter einem OneDrive-Wurzelordner? Windows leitet
/// „Dokumente“ oft dorthin um; Exporte mit Screenshots würden dann
/// ungefragt in die Cloud synchronisiert.
pub fn is_cloud_synced(path: &Path) -> bool {
    let roots: Vec<PathBuf> = ["OneDrive", "OneDriveCommercial", "OneDriveConsumer"]
        .iter()
        .filter_map(std::env::var_os)
        .filter(|root| !root.is_empty())
        .map(PathBuf::from)
        .collect();
    is_under_any(path, &roots)
}

/// Komponentenweiser Präfixvergleich ohne Groß-/Kleinschreibung (Windows-Pfade).
fn is_under_any(path: &Path, roots: &[PathBuf]) -> bool {
    let lower = |p: &Path| PathBuf::from(p.to_string_lossy().to_lowercase());
    let path = lower(path);
    roots.iter().any(|root| path.starts_with(lower(root)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_contents_sot() {
        let s = Settings {
            language: "de".to_owned(),
            ..Settings::default()
        };
        assert_eq!(s.hotkey, "Ctrl+Shift+R");
        assert_eq!(s.pause_hotkey, "Ctrl+Shift+P");
        assert_eq!(s.export_format, "html");
        assert!(s.open_after_export);
        assert_eq!(s.theme, "system");
        assert!(s.default_export_folder.is_empty());
        assert!(s.guides_folder.is_empty());
        assert_eq!(Settings::default().language, SYSTEM);
    }

    #[test]
    fn language_follows_windows_until_a_code_is_chosen() {
        let system = Settings::default();
        assert_eq!(system.locale(), Locale::system());
        for code in ["de", "fr", "nl"] {
            let chosen = Settings {
                language: code.to_owned(),
                ..Settings::default()
            };
            assert_eq!(chosen.locale().code(), code, "stored codes stay valid");
        }
        let old: Settings = serde_json::from_str(r#"{"language":"it"}"#).expect("parse");
        assert_eq!(old.locale().code(), "it");
    }

    #[test]
    fn changes_are_clamped_and_normalized() {
        let mut s = Settings::default();
        s.apply(SettingChange::TypingPauseMs(1));
        assert_eq!(s.typing_pause_ms, LIMITS.typing_pause_ms.min);
        s.apply(SettingChange::ScrollPauseMs(u64::MAX));
        assert_eq!(s.scroll_pause_ms, LIMITS.scroll_pause_ms.max);
        s.apply(SettingChange::PdfMarginMm(0));
        assert_eq!(s.pdf_margin_mm, LIMITS.pdf_margin_mm.min);
        s.apply(SettingChange::AnnotationStroke(f64::NAN));
        assert!((s.annotation_stroke - marks().stroke).abs() < f64::EPSILON);
        s.apply(SettingChange::AnnotationStroke(99.0));
        assert!((s.annotation_stroke - LIMITS.annotation_stroke.max).abs() < f64::EPSILON);

        s.apply(SettingChange::AnnotationColor(" #2563eb ".to_owned()));
        assert_eq!(s.annotation_color, "#2563EB");
        s.apply(SettingChange::AnnotationColor("red".to_owned()));
        assert_eq!(s.annotation_color, "#2563EB", "invalid colors are ignored");

        s.apply(SettingChange::PdfPaper("LETTER".to_owned()));
        assert_eq!(s.pdf_paper, "letter");
        s.apply(SettingChange::PdfPaper("a5".to_owned()));
        assert_eq!(s.pdf_paper, "a4");

        s.apply(SettingChange::FileNameTemplate("  ".to_owned()));
        assert_eq!(s.file_name_template, DEFAULT_FILE_NAME_TEMPLATE);
        s.apply(SettingChange::FileNameTemplate(
            "..\\{date}/{title}".to_owned(),
        ));
        assert_eq!(s.file_name_template, ".. {date} {title}");
        s.apply(SettingChange::TitleTemplate("  ".to_owned()));
        assert_eq!(s.title_template(), None);
        s.apply(SettingChange::TitleTemplate(" {app} how-to ".to_owned()));
        assert_eq!(s.title_template(), Some("{app} how-to"));
    }

    #[test]
    fn brand_and_accent_fall_back_to_the_preset() {
        let mut s = Settings::default();
        assert_eq!(s.brand, brand::default_brand());
        assert_eq!(s.accent_color, "");
        let other = &brand::brands()[1].code;
        s.apply(SettingChange::Brand(format!(" {other} ")));
        assert_eq!(&s.brand, other);
        s.apply(SettingChange::Brand("neon".to_owned()));
        assert_eq!(s.brand, brand::default_brand(), "unknown presets fall back");
        s.apply(SettingChange::AccentColor("#12ab34".to_owned()));
        assert_eq!(s.accent_color, "#12AB34");
        s.apply(SettingChange::AccentColor("teal".to_owned()));
        assert_eq!(s.accent_color, "#12AB34", "invalid colors are ignored");
        s.apply(SettingChange::AccentColor(String::new()));
        assert_eq!(s.accent_color, "", "empty means the preset accent");
    }

    #[test]
    fn export_style_follows_brand_accent_and_credit() {
        let mut s = Settings::default();
        assert!(s.export_credit);
        assert_eq!(s.export_style(), ExportStyle::default());
        let other = brand::brands()[1].code.clone();
        s.apply(SettingChange::Brand(other.clone()));
        s.apply(SettingChange::AccentColor("#aa3366".to_owned()));
        s.apply(SettingChange::ExportCredit(false));
        assert_eq!(
            s.export_style(),
            ExportStyle {
                brand: other,
                accent: "#AA3366".to_owned(),
                credit: false,
            }
        );
        s.reset_advanced();
        assert!(!s.export_credit, "credit is not an advanced setting");
        s.apply(SettingChange::ExportCredit(true));
        assert!(s.export_credit);
    }

    #[test]
    fn lowering_the_recent_limit_trims_the_list() {
        let mut s = Settings {
            recent_paths: (0..10).map(|i| format!("/g{i}.steps")).collect(),
            ..Settings::default()
        };
        s.apply(SettingChange::RecentLimit(3));
        assert_eq!(s.recent_paths, ["/g0.steps", "/g1.steps", "/g2.steps"]);
        s.apply(SettingChange::RecentLimit(0));
        assert_eq!(s.recent_limit, 1);
    }

    #[test]
    fn reset_advanced_restores_defaults_and_keeps_the_rest() {
        let mut s = Settings {
            language: "fr".to_owned(),
            pdf_paper: "letter".to_owned(),
            annotation_color: "#2563EB".to_owned(),
            typing_pause_ms: 800,
            recent_limit: 20,
            title_template: "{app}".to_owned(),
            file_name_template: "{date} {title}".to_owned(),
            pdf_margin_mm: 30,
            annotation_stroke: 5.0,
            ..Settings::default()
        };
        s.reset_advanced();
        let defaults = Settings::default();
        assert_eq!(s.typing_pause_ms, defaults.typing_pause_ms);
        assert_eq!(s.recent_limit, defaults.recent_limit);
        assert_eq!(s.title_template, "");
        assert_eq!(s.file_name_template, DEFAULT_FILE_NAME_TEMPLATE);
        assert_eq!(s.pdf_margin_mm, defaults.pdf_margin_mm);
        assert!((s.annotation_stroke - defaults.annotation_stroke).abs() < f64::EPSILON);
        assert_eq!(s.language, "fr");
        assert_eq!(s.pdf_paper, "letter");
        assert_eq!(s.annotation_color, "#2563EB");
    }

    #[test]
    fn load_repairs_hand_edited_values() {
        let dir = scratch_dir("sanitize");
        std::fs::write(
            settings_path(&dir),
            r#"{"typing_pause_ms":1,"recent_limit":500,"pdf_paper":"tabloid","annotation_color":"blue","file_name_template":"a/b","brand":"neon","accent_color":"teal"}"#,
        )
        .expect("settings");
        let s = load(&dir);
        assert_eq!(s.typing_pause_ms, LIMITS.typing_pause_ms.min);
        assert_eq!(s.recent_limit, LIMITS.recent_limit.max);
        assert_eq!(s.pdf_paper, "a4");
        assert_eq!(s.annotation_color, marks().color);
        assert_eq!(s.file_name_template, "a b");
        assert_eq!(s.brand, brand::default_brand());
        assert_eq!(s.accent_color, "");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn recording_shortcuts_need_ctrl_alt_or_win() {
        for chord in ["Ctrl+Shift+R", "Ctrl+S", "Alt+F9", "Win+R", "ctrl+shift+p"] {
            assert!(is_recording_shortcut(chord), "{chord}");
        }
        for chord in ["Shift+F9", "F9", "shift+a", "Ctrl", "Shift", ""] {
            assert!(!is_recording_shortcut(chord), "{chord}");
        }
    }

    #[test]
    fn load_replaces_shortcuts_without_ctrl_alt_or_win() {
        let dir = scratch_dir("hotkeys");
        std::fs::write(
            settings_path(&dir),
            r#"{"hotkey":"Shift+F9","pause_hotkey":"F9"}"#,
        )
        .expect("settings");
        let s = load(&dir);
        assert_eq!(s.hotkey, "Ctrl+Shift+R");
        assert_eq!(s.pause_hotkey, "Ctrl+Shift+P");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn save_replaces_settings_and_leaves_no_temp_file() {
        let dir = scratch_dir("save");
        let first = Settings {
            language: "de".to_owned(),
            ..Settings::default()
        };
        save(&dir, &first).expect("first save");
        let second = Settings {
            language: "fr".to_owned(),
            ..Settings::default()
        };
        save(&dir, &second).expect("second save");
        assert_eq!(load(&dir).language, "fr");
        let mut names: Vec<String> = std::fs::read_dir(&dir)
            .expect("read dir")
            .map(|entry| {
                entry
                    .expect("entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect();
        names.sort();
        assert_eq!(names, ["settings.json"]);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn serde_keeps_unknown_old_files() {
        let raw = r#"{"hotkey":"Ctrl+Shift+R","pause_hotkey":"Ctrl+Shift+P","language":"de"}"#;
        let s: Settings = serde_json::from_str(raw).expect("parse");
        assert_eq!(s.export_format, "html");
        assert!(s.open_after_export);
        assert_eq!(s.theme, "system");
        assert!(s.export_credit, "older files keep the credit on");
    }

    #[test]
    fn normalize_helpers() {
        assert_eq!(normalize_theme("Dunkel"), "dark");
        assert_eq!(normalize_export_format(".steps"), "steps");
        assert_eq!(normalize_export_format("PDF"), "pdf");
        assert_eq!(normalize_export_format("docx"), "html");
        assert_eq!(normalize_language("EN"), "en");
        assert_eq!(normalize_language("fr-CA"), "fr");
        assert_eq!(normalize_language("klingon"), "en");
        assert_eq!(normalize_language(" System "), SYSTEM);
    }

    #[test]
    fn resolve_folder_falls_back_on_junk() {
        let fallback = PathBuf::from("/tmp");
        let (path, used_fallback) =
            resolve_folder("/definitely/not/a/real/path/xyz-gro190", fallback.clone());
        assert!(used_fallback);
        assert_eq!(path, fallback);
    }

    fn scratch_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("hows-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    #[test]
    fn cloud_sync_detection_matches_components_case_insensitively() {
        let roots = vec![PathBuf::from("C:/Users/Ada/OneDrive - Contoso")];
        assert!(is_under_any(
            Path::new("c:/users/ada/onedrive - contoso/Documents"),
            &roots
        ));
        assert!(!is_under_any(
            Path::new("C:/Users/Ada/OneDrive - Contoso Backup/Documents"),
            &roots
        ));
        assert!(!is_under_any(Path::new("C:/Users/Ada/Documents"), &roots));
        assert!(!is_under_any(Path::new("C:/Users/Ada/Documents"), &[]));
    }
}
