//! Exporte über den Daten einer `.steps`-Datei (Spec §6).
//!
//! Markdown schreibt einen Ordner `images/` für GitHub-Issues und Wikis.
//! HTML ist eine selbst-enthaltene Datei, die Bilder als Base64 enthält.
//! JSON ist für Skripte und Ticket-Systeme lesbar.
//! PDF ist A4 zum Drucken und Ablegen, ohne Webview, auch in der CLI.
//!
//! Beide Output-Modi ([`ExportMode::Sop`] und [`ExportMode::BugReport`])
//! sind Templates über denselben Daten.
//!
//! Overlays werden beim HTML-/Markdown-/PDF-Export in eine
//! PNG-Kopie eingebrannt; Original-Screenshots in `.steps` bleiben sauber.
//!
//! HTML und PDF tragen den Markenlook der App ([`ExportStyle`]): Farben aus
//! `brands.json`, die Markenschrift eingebettet und optional den Hinweis
//! „Erstellt mit Hows“.

pub mod brand;
mod font;
mod overlay;
mod pdf;
mod util;

pub use pdf::{pdf, write_pdf_export, Paper, PdfPage, DEFAULT_MARGIN_MM};

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::io::Cursor;
use std::path::{Component, Path};

use brand::{Palette, Skin};
use image::ImageFormat;
use overlay::present_png;
use steps_i18n::{Key, Locale};
use steps_store::{Action, Guide, Step, StepsFile};
use util::{base64_encode, format_utc, html_escape};

/// Aussehen eines Dokument-Exports (HTML, PDF, Markdown).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportStyle {
    /// Code aus `brands.json`; unbekannte Codes ergeben die Standardvariante.
    pub brand: String,
    /// Eigener Akzent `#RRGGBB`; leer oder ungültig = Akzent der Variante.
    pub accent: String,
    /// Hinweis „Erstellt mit Hows“ am Ende des Exports.
    pub credit: bool,
}

impl Default for ExportStyle {
    fn default() -> Self {
        Self {
            brand: brand::default_brand().to_owned(),
            accent: String::new(),
            credit: true,
        }
    }
}

impl ExportStyle {
    /// Palette des Exports, mit demselben Akzent wie in der App.
    pub fn palette(&self, skin: Skin) -> Palette {
        brand::brand_palette(&self.brand, skin, &self.accent)
    }

    /// Hinweistext in der Sprache der Anleitung, `None` wenn abgeschaltet.
    fn credit_text(&self, lang: Locale) -> Option<&'static str> {
        self.credit.then(|| lang.text(Key::ExportCredit))
    }
}

/// PNG-Bytes eines Schritts für Raster-Exporte (Zuschnitt und Overlays).
fn export_png<'a>(step: &Step, original: &'a [u8]) -> Cow<'a, [u8]> {
    present_png(original, step.crop, &step.overlays)
}

/// Output-Modus: gleiche Daten, unterschiedliche Templates (Spec §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportMode {
    /// Sauberer Guide, technische Details ausgeblendet.
    Sop,
    /// Inklusive OS-Version, App-Namen, Zeitstempeln, Element-Metadaten.
    BugReport,
}

fn language(guide: &Guide) -> Locale {
    Locale::resolve(&guide.language)
}

fn title_or_default(guide: &Guide, lang: Locale) -> String {
    if guide.title.is_empty() {
        lang.text(Key::ExportUntitled).to_owned()
    } else {
        guide.title.clone()
    }
}

/// `Beschriftung: Wert` mit der Zeichensetzung der Sprache (Französisch
/// setzt ein schmales geschütztes Leerzeichen vor den Doppelpunkt).
fn labeled(lang: Locale, key: Key, value: &str) -> String {
    lang.fill(
        Key::ExportLabel,
        &[("label", lang.text(key)), ("value", value)],
    )
}

/// Dateiname eines Screenshots im `images/`-Ordner des Markdown-Exports.
fn image_file_name(step: &Step) -> String {
    format!("{}.png", step.id)
}

/// Technische Metadaten-Zeilen eines Schritts (nur Bug-Report-Modus).
fn step_metadata_lines(step: &Step, lang: Locale) -> Vec<String> {
    let mut lines = Vec::new();
    lines.push(labeled(
        lang,
        Key::ExportTime,
        &format_utc(step.timestamp_ms),
    ));
    if let Some(element) = &step.element {
        let mut parts = Vec::new();
        if let Some(name) = &element.name {
            parts.push(name.clone());
        }
        if let Some(control_type) = &element.control_type {
            parts.push(control_type.clone());
        }
        if let Some(automation_id) = &element.automation_id {
            parts.push(format!("AutomationId={automation_id}"));
        }
        if !parts.is_empty() {
            lines.push(labeled(lang, Key::ExportElement, &parts.join(", ")));
        }
        if let Some(app) = &element.app_name {
            lines.push(labeled(lang, Key::ExportApp, app));
        }
        if let Some(window) = &element.window_title {
            lines.push(labeled(lang, Key::ExportWindow, window));
        }
    }
    if let Some(position) = step.position {
        lines.push(labeled(
            lang,
            Key::ExportPosition,
            &format!("({}, {})", position.x, position.y),
        ));
    }
    if let Some(monitor) = &step.monitor {
        let scale = labeled(
            lang,
            Key::ExportScale,
            &format!("{}%", (monitor.scale_factor * 100.0).round()),
        );
        lines.push(labeled(
            lang,
            Key::ExportMonitor,
            &format!(
                "{} ({}×{}, {scale})",
                monitor.index, monitor.width, monitor.height
            ),
        ));
    }
    lines
}

// --- Markdown ---------------------------------------------------------------

/// Rendert den Guide als Markdown. Bilder werden als `images/{step-id}.png`
/// referenziert; die zugehörigen Dateien liefert [`markdown_images`] bzw.
/// schreibt [`write_markdown_export`]. Markdown kennt keine Farben; vom
/// Stil gilt nur der Hinweis am Ende.
pub fn markdown(file: &StepsFile, mode: ExportMode, style: &ExportStyle) -> String {
    let guide = &file.guide;
    let lang = language(guide);
    let mut out = String::new();

    let _ = writeln!(out, "# {}", title_or_default(guide, lang));
    out.push('\n');
    if let Some(description) = &guide.description {
        let _ = writeln!(out, "{description}");
        out.push('\n');
    }
    if mode == ExportMode::BugReport {
        let _ = writeln!(
            out,
            "*{}*  ",
            labeled(lang, Key::ExportCreated, &format_utc(guide.created_at_ms))
        );
        if let Some(environment) = &guide.environment {
            let mut parts = Vec::new();
            if let Some(os) = &environment.os {
                parts.push(os.clone());
            }
            if let Some(version) = &environment.os_version {
                parts.push(version.clone());
            }
            if !parts.is_empty() {
                let _ = writeln!(
                    out,
                    "*{}*  ",
                    labeled(lang, Key::ExportEnvironment, &parts.join(" "))
                );
            }
        }
        out.push('\n');
    }

    for (number, step) in guide.steps.iter().enumerate() {
        let _ = writeln!(out, "{}. {}", number + 1, step.text.effective());
        if file.images.contains_key(&step.id) {
            let _ = writeln!(
                out,
                "\n   ![{} {}](images/{})",
                lang.text(Key::ExportStep),
                number + 1,
                image_file_name(step)
            );
        }
        if mode == ExportMode::BugReport {
            for line in step_metadata_lines(step, lang) {
                let _ = writeln!(out, "   - {line}");
            }
        }
        out.push('\n');
    }
    if let Some(credit) = style.credit_text(lang) {
        let _ = writeln!(out, "---\n\n*{credit}*");
    }
    out
}

/// Bilddateien des Markdown-Exports. Pfad relativ zum Dokument, dazu die PNG-Bytes.
/// Overlays sind bereits eingebrannt. Der Aufrufer besitzt die Bytes.
pub fn markdown_images(file: &StepsFile) -> BTreeMap<String, Vec<u8>> {
    file.guide
        .steps
        .iter()
        .filter_map(|step| {
            file.images.get(&step.id).map(|png| {
                (
                    format!("images/{}", image_file_name(step)),
                    export_png(step, png).into_owned(),
                )
            })
        })
        .collect()
}

/// Schreibt den Markdown-Export als `guide.md` + `images/`-Ordner in ein
/// Verzeichnis.
pub fn write_markdown_export(
    file: &StepsFile,
    mode: ExportMode,
    style: &ExportStyle,
    directory: impl AsRef<Path>,
) -> std::io::Result<()> {
    let directory = directory.as_ref();
    let images = markdown_images(file);
    if let Some(escaping) = images.keys().find(|path| !stays_inside(path)) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("{escaping} would leave the export folder"),
        ));
    }
    std::fs::create_dir_all(directory)?;
    std::fs::write(directory.join("guide.md"), markdown(file, mode, style))?;
    if !images.is_empty() {
        std::fs::create_dir_all(directory.join("images"))?;
        for (relative_path, png) in images {
            std::fs::write(directory.join(relative_path), png)?;
        }
    }
    Ok(())
}

/// `directory.join(relative)` bleibt im Ordner: nur normale Pfadteile, also
/// kein `..`, kein Laufwerk und keine Wurzel (unter Windows auch mit `\`).
fn stays_inside(relative: &str) -> bool {
    Path::new(relative)
        .components()
        .all(|part| matches!(part, Component::Normal(_)))
}

// --- HTML --------------------------------------------------------------------

/// Schreibt den HTML-Export als eine selbst-enthaltene Datei.
pub fn write_html_export(
    file: &StepsFile,
    mode: ExportMode,
    style: &ExportStyle,
    path: impl AsRef<Path>,
) -> std::io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(path, html(file, mode, style))
}

/// Rendert den Guide als eine selbst-enthaltene HTML-Datei; Bilder und
/// Markenschrift sind base64-inline eingebettet (keine externen Referenzen,
/// kein Netzwerk). Die Seite folgt dem Farbschema des Betrachters, gedruckt
/// wird immer hell.
pub fn html(file: &StepsFile, mode: ExportMode, style: &ExportStyle) -> String {
    let guide = &file.guide;
    let lang = language(guide);
    let title = html_escape(&title_or_default(guide, lang));
    let mut out = String::new();

    let _ = write!(
        out,
        "<!doctype html>\n<html lang=\"{}\">\n<head>\n<meta charset=\"utf-8\">\n\
         <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
         <title>{title}</title>\n<style>{}</style>\n</head>\n<body>\n<main>\n",
        html_escape(&guide.language),
        html_style(style),
    );
    let _ = writeln!(out, "<h1>{title}</h1>");
    if let Some(description) = &guide.description {
        let _ = writeln!(
            out,
            "<p class=\"description\">{}</p>",
            html_escape(description)
        );
    }
    if mode == ExportMode::BugReport {
        let _ = writeln!(
            out,
            "<p class=\"meta\">{}</p>",
            labeled(lang, Key::ExportCreated, &format_utc(guide.created_at_ms))
        );
        if let Some(environment) = &guide.environment {
            let mut parts = Vec::new();
            if let Some(os) = &environment.os {
                parts.push(html_escape(os));
            }
            if let Some(version) = &environment.os_version {
                parts.push(html_escape(version));
            }
            if !parts.is_empty() {
                let _ = writeln!(
                    out,
                    "<p class=\"meta\">{}</p>",
                    labeled(lang, Key::ExportEnvironment, &parts.join(" "))
                );
            }
        }
    }

    let _ = writeln!(out, "<ol class=\"steps\">");
    for (number, step) in guide.steps.iter().enumerate() {
        let _ = writeln!(
            out,
            "<li class=\"step\">\n<span class=\"num\" aria-hidden=\"true\">{}</span>\n\
             <div class=\"body\">\n<p class=\"text\">{}</p>",
            number + 1,
            html_escape(step.text.effective())
        );
        if let Some(png) = file.images.get(&step.id) {
            let burned = export_png(step, png);
            let _ = writeln!(
                out,
                "<img alt=\"{} {}\" src=\"{}\">",
                lang.text(Key::ExportStep),
                number + 1,
                image_data_uri(&burned)
            );
        }
        if mode == ExportMode::BugReport {
            let _ = writeln!(out, "<ul class=\"meta\">");
            for line in step_metadata_lines(step, lang) {
                let _ = writeln!(out, "<li>{}</li>", html_escape(&line));
            }
            let _ = writeln!(out, "</ul>");
        }
        let _ = writeln!(out, "</div>\n</li>");
    }
    let _ = writeln!(out, "</ol>");
    if let Some(credit) = style.credit_text(lang) {
        let _ = writeln!(
            out,
            "<footer class=\"credit\">{}</footer>",
            html_escape(credit)
        );
    }
    let _ = writeln!(out, "</main>\n</body>\n</html>");
    out
}

/// Web-Schnitte der Markenschrift (variabel, 200 bis 800) mit ihren
/// Unicode-Bereichen, wie das Frontend sie über fontsource lädt.
const HTML_FONTS: [(&[u8], &str); 2] = [
    (
        include_bytes!("../fonts/atkinson-hyperlegible-next-latin-wght-normal.woff2"),
        "U+0000-00FF,U+0131,U+0152-0153,U+02BB-02BC,U+02C6,U+02DA,U+02DC,U+0304,U+0308,\
         U+0329,U+2000-206F,U+20AC,U+2122,U+2191,U+2193,U+2212,U+2215,U+FEFF,U+FFFD",
    ),
    (
        include_bytes!("../fonts/atkinson-hyperlegible-next-latin-ext-wght-normal.woff2"),
        "U+0100-02BA,U+02BD-02C5,U+02C7-02CC,U+02CE-02D7,U+02DD-02FF,U+0304,U+0308,U+0329,\
         U+1D00-1DBF,U+1E00-1E9F,U+1EF2-1EFF,U+2020,U+20A0-20AB,U+20AD-20C0,U+2113,\
         U+2C60-2C7F,U+A720-A7FF",
    ),
];

/// CSS-Variablen `--sl-<token>` einer Palette, wie `paletteVariables` in der App.
fn palette_variables(palette: &Palette) -> String {
    let mut out = String::new();
    for (token, value) in palette.tokens() {
        let _ = write!(out, "--sl-{}:{value};", token.replace('_', "-"));
    }
    out
}

fn html_style(style: &ExportStyle) -> String {
    let font = brand::font();
    let mut css = String::new();
    for (woff2, range) in HTML_FONTS {
        let _ = write!(
            css,
            "@font-face{{font-family:\"{}\";font-style:normal;font-weight:200 800;\
             font-display:swap;src:url(data:font/woff2;base64,{}) format(\"woff2\");\
             unicode-range:{range}}}",
            font.family,
            base64_encode(woff2),
        );
    }
    let _ = write!(
        css,
        ":root{{{}--sl-font:{};color-scheme:light}}\
         @media screen and (prefers-color-scheme:dark){{:root{{{}color-scheme:dark}}}}{HTML_LAYOUT}",
        palette_variables(&style.palette(Skin::Light)),
        font.css_stack(),
        palette_variables(&style.palette(Skin::Dark)),
    );
    css
}

/// Layout des HTML-Exports. Die Datei ist eigenständig und lädt kein CSS der
/// App, deshalb stehen die Radien als Zahl hier: 12 px für Schrittkarten und
/// 8 px für Bilder wie `--sl-radius-lg` und `--sl-radius` in
/// `app/src/styles/tokens.css`, 999 px wie `--sl-radius-pill`. Ein Test in
/// `tests/export.rs` hält jeden Radius auf dieser Skala.
const HTML_LAYOUT: &str = "\
*{box-sizing:border-box}\
body{margin:0;background:var(--sl-bg);color:var(--sl-ink);font:16px/1.55 var(--sl-font)}\
main{max-width:52rem;margin:0 auto;padding:2.5rem 1.25rem 3rem}\
h1{font-size:1.9rem;line-height:1.2;margin:0 0 .75rem;padding-bottom:.75rem;\
border-bottom:3px solid var(--sl-accent)}\
.description{margin:0 0 .5rem;color:var(--sl-muted);font-size:1.05rem}\
.meta{margin:.25rem 0;color:var(--sl-muted);font-size:.9rem}\
.steps{list-style:none;margin:2rem 0 0;padding:0;display:grid;gap:1rem}\
.step{display:flex;gap:1rem;padding:1.1rem;background:var(--sl-raised);\
border:1px solid var(--sl-line);border-radius:12px;break-inside:avoid}\
.num{flex:none;display:grid;place-items:center;width:2rem;height:2rem;border-radius:999px;\
background:var(--sl-accent);color:var(--sl-on-accent);font-weight:700;\
font-variant-numeric:tabular-nums;-webkit-print-color-adjust:exact;print-color-adjust:exact}\
.body{flex:1;min-width:0}\
.text{margin:.2rem 0 0;font-size:1.05rem}\
.step img{display:block;max-width:100%;height:auto;margin-top:.75rem;\
border:1px solid var(--sl-line);border-radius:8px}\
ul.meta{margin:.6rem 0 0;padding-left:1.1rem}\
.credit{margin-top:2.5rem;text-align:center;color:var(--sl-muted);font-size:.85rem}\
@media print{body{background:none}main{max-width:none;padding:0}}";

// --- JSON --------------------------------------------------------------------

/// Rendert den Guide als maschinenlesbares JSON (pretty-printed).
///
/// Der SOP-Modus enthält nur Anzeige-Daten (Nummer, Aktion, effektiver Text,
/// Bildreferenz); der Bug-Report-Modus zusätzlich Umgebung, Zeitstempel,
/// Position, Monitor und Element-Metadaten.
pub fn json(file: &StepsFile, mode: ExportMode) -> String {
    let guide = &file.guide;
    let steps: Vec<serde_json::Value> = guide
        .steps
        .iter()
        .enumerate()
        .map(|(number, step)| {
            let mut value = serde_json::json!({
                "number": number + 1,
                "id": step.id,
                "action": action_name(&step.action),
                "text": step.text.effective(),
                "image": file
                    .images
                    .contains_key(&step.id)
                    .then(|| format!("images/{}", image_file_name(step))),
                "overlays": step.overlays,
            });
            if let (ExportMode::BugReport, Some(object)) = (mode, value.as_object_mut()) {
                object.insert(
                    "action_detail".to_owned(),
                    serde_json::to_value(&step.action).unwrap_or(serde_json::Value::Null),
                );
                object.insert("timestamp_ms".to_owned(), step.timestamp_ms.into());
                object.insert(
                    "position".to_owned(),
                    serde_json::to_value(step.position).unwrap_or(serde_json::Value::Null),
                );
                object.insert(
                    "monitor".to_owned(),
                    serde_json::to_value(&step.monitor).unwrap_or(serde_json::Value::Null),
                );
                object.insert(
                    "element".to_owned(),
                    serde_json::to_value(&step.element).unwrap_or(serde_json::Value::Null),
                );
            }
            value
        })
        .collect();

    let mut root = serde_json::json!({
        "schema_version": guide.schema_version,
        "mode": match mode {
            ExportMode::Sop => "sop",
            ExportMode::BugReport => "bug_report",
        },
        "title": guide.title,
        "description": guide.description,
        "language": guide.language,
        "steps": steps,
    });
    if let (ExportMode::BugReport, Some(object)) = (mode, root.as_object_mut()) {
        object.insert("created_at_ms".to_owned(), guide.created_at_ms.into());
        object.insert(
            "environment".to_owned(),
            serde_json::to_value(&guide.environment).unwrap_or(serde_json::Value::Null),
        );
    }
    serde_json::to_string_pretty(&root).unwrap_or_else(|_| "{}".to_owned())
}

/// `data:`-URI eines PNG-Screenshots (für HTML-Export und Editor-Anzeige).
pub fn image_data_uri(png: &[u8]) -> String {
    format!("data:image/png;base64,{}", base64_encode(png))
}

/// Verkleinerte Kopie eines Screenshots als `data:`-URI, höchstens `max_width`
/// Pixel breit und im Seitenverhältnis des Originals. `None` für kein lesbares PNG.
pub fn thumbnail_data_uri(png: &[u8], max_width: u32) -> Option<String> {
    let original = image::load_from_memory_with_format(png, ImageFormat::Png).ok()?;
    let thumbnail = if original.width() > max_width {
        original.thumbnail(max_width, u32::MAX)
    } else {
        original
    };
    let mut encoded = Vec::new();
    thumbnail
        .write_to(&mut Cursor::new(&mut encoded), ImageFormat::Png)
        .ok()?;
    Some(image_data_uri(&encoded))
}

/// Kurzer Aktionsname für den JSON-Export.
fn action_name(action: &Action) -> &'static str {
    match action {
        Action::Click => "click",
        Action::DoubleClick => "double_click",
        Action::RightClick => "right_click",
        Action::KeyCombo { .. } => "key_combo",
        Action::Scroll { .. } => "scroll",
        Action::TextInput => "text_input",
        Action::ManualNote => "manual_note",
    }
}
