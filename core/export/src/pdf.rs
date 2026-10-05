//! PDF-Export ohne Webview: ein kleiner PDF-1.4-Writer mit eingebetteter
//! Markenschrift und Flate-komprimierten RGB-Bildern. Keine externen
//! Ressourcen, kein Netzwerk; dieselben Garantien wie der HTML-Export.
//!
//! Die Schrift steckt als `CIDFontType2` mit `Identity-H` im Dokument, Text
//! steht als Glyph-IDs in den Inhaltsströmen, und eine `ToUnicode`-Tabelle
//! macht ihn wieder durchsuch- und kopierbar. Damit geht jedes Zeichen, das
//! die Schrift kennt, auch außerhalb von `WinAnsi` (etwa Polnisch oder
//! Tschechisch). Farben kommen aus der hellen Palette des [`ExportStyle`].

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use steps_store::StepsFile;

use crate::brand::{self, Palette, Skin};
use crate::font::Font;
use crate::util::format_utc;
use crate::{
    export_png, labeled, language, step_metadata_lines, title_or_default, ExportMode, ExportStyle,
};
use steps_i18n::{Key, Locale};

/// Papierformat, hochkant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Paper {
    /// ISO A4, 210 × 297 mm.
    #[default]
    A4,
    /// US Letter, 8.5 × 11 in.
    Letter,
}

impl Paper {
    /// Alle Formate, etwa für eine Auswahlliste.
    pub const ALL: [Paper; 2] = [Paper::A4, Paper::Letter];

    /// Kurzname für Einstellungen (`a4` | `letter`).
    pub fn code(self) -> &'static str {
        match self {
            Paper::A4 => "a4",
            Paper::Letter => "letter",
        }
    }

    /// Format zum Kurznamen, ohne Groß/Klein.
    pub fn from_code(code: &str) -> Option<Paper> {
        Self::ALL
            .into_iter()
            .find(|paper| paper.code().eq_ignore_ascii_case(code.trim()))
    }

    /// Breite und Höhe in PostScript-Punkten.
    fn size(self) -> (f32, f32) {
        match self {
            Paper::A4 => (595.28, 841.89),
            Paper::Letter => (612.0, 792.0),
        }
    }
}

/// Standard-Seitenrand in Millimetern.
pub const DEFAULT_MARGIN_MM: u32 = 18;

const POINTS_PER_MM: f32 = 72.0 / 25.4;

/// Seite des PDF-Exports: Format und Rand.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PdfPage {
    /// Papierformat.
    pub paper: Paper,
    /// Rand auf allen Seiten in ganzen Millimetern.
    pub margin_mm: u32,
}

impl Default for PdfPage {
    fn default() -> Self {
        Self {
            paper: Paper::default(),
            margin_mm: DEFAULT_MARGIN_MM,
        }
    }
}

/// Seitenmaße in Punkten, aus einer [`PdfPage`] berechnet.
#[derive(Debug, Clone, Copy)]
struct Geometry {
    width: f32,
    height: f32,
    margin: f32,
}

impl Geometry {
    /// Der Rand bleibt höchstens ein Viertel der Breite, damit Platz für
    /// Text und Bilder übrig ist.
    fn new(page: PdfPage) -> Self {
        let (width, height) = page.paper.size();
        Self {
            width,
            height,
            margin: (f32::from(u16::try_from(page.margin_mm).unwrap_or(u16::MAX)) * POINTS_PER_MM)
                .min(width / 4.0),
        }
    }

    fn content_width(self) -> f32 {
        self.width - 2.0 * self.margin
    }

    fn top(self) -> f32 {
        self.height - self.margin
    }
}

/// Einzug des Schritt-Texts hinter der Nummer.
const STEP_INDENT: f32 = 28.0;
/// Durchmesser des Nummernkreises vor jedem Schritt.
const BADGE: f32 = 18.0;
/// Platz für Seitenzahl und Hinweis am unteren Rand.
const FOOTER_HEIGHT: f32 = 20.0;
/// Ein Screenshot nimmt höchstens diesen Anteil der Seitenhöhe ein.
const MAX_IMAGE_SHARE: f32 = 0.55;

#[derive(Clone, Copy)]
enum Face {
    Regular,
    Bold,
}

impl Face {
    const ALL: [Face; 2] = [Face::Regular, Face::Bold];

    fn resource(self) -> &'static str {
        match self {
            Face::Regular => "F1",
            Face::Bold => "F2",
        }
    }

    fn font(self) -> &'static Font {
        match self {
            Face::Regular => Font::regular(),
            Face::Bold => Font::bold(),
        }
    }

    fn index(self) -> usize {
        match self {
            Face::Regular => 0,
            Face::Bold => 1,
        }
    }

    /// Strichstärke für den Font-Deskriptor, grob nach Schnitt.
    fn stem(self) -> u32 {
        match self {
            Face::Regular => 80,
            Face::Bold => 140,
        }
    }
}

/// RGB-Farbe 0..1 für die PDF-Farboperatoren.
#[derive(Debug, Clone, Copy)]
struct Rgb([f32; 3]);

impl Rgb {
    fn of(hex: &str) -> Rgb {
        Rgb(brand::rgb_unit(hex).unwrap_or_default())
    }

    fn fill(self) -> String {
        let [r, g, b] = self.0;
        format!("{r:.3} {g:.3} {b:.3} rg")
    }

    fn stroke(self) -> String {
        let [r, g, b] = self.0;
        format!("{r:.3} {g:.3} {b:.3} RG")
    }
}

/// Die Farben, die das PDF aus der Palette nutzt.
struct Colors {
    ink: Rgb,
    muted: Rgb,
    accent: Rgb,
    on_accent: Rgb,
    line: Rgb,
}

impl Colors {
    fn new(palette: &Palette) -> Self {
        Self {
            ink: Rgb::of(&palette.ink),
            muted: Rgb::of(&palette.muted),
            accent: Rgb::of(&palette.accent),
            on_accent: Rgb::of(&palette.on_accent),
            line: Rgb::of(&palette.line),
        }
    }
}

struct PdfImage {
    width: u32,
    height: u32,
    /// zlib-komprimierte RGB-Pixel (8 Bit je Kanal).
    data: Vec<u8>,
}

/// Seitenweiser Layout-Zustand; `y` läuft von oben nach unten.
struct Layout {
    geometry: Geometry,
    colors: Colors,
    pages: Vec<String>,
    current: String,
    y: f32,
    images: Vec<PdfImage>,
    /// Benutzte Glyphen je Schnitt: ID auf das Zeichen, für das sie steht.
    used: [BTreeMap<u16, char>; 2],
}

impl Layout {
    fn new(geometry: Geometry, colors: Colors) -> Self {
        Self {
            geometry,
            colors,
            pages: Vec::new(),
            current: String::new(),
            y: geometry.top(),
            images: Vec::new(),
            used: [BTreeMap::new(), BTreeMap::new()],
        }
    }

    fn bottom(&self) -> f32 {
        self.geometry.margin + FOOTER_HEIGHT
    }

    /// Höhe, die auf einer leeren Seite für Inhalt bleibt.
    fn usable_height(&self) -> f32 {
        self.geometry.top() - self.bottom()
    }

    fn at_page_top(&self) -> bool {
        (self.y - self.geometry.top()).abs() < f32::EPSILON
    }

    fn new_page(&mut self) {
        self.pages.push(std::mem::take(&mut self.current));
        self.y = self.geometry.top();
    }

    /// Bricht um, wenn `height` nicht mehr auf die Seite passt.
    fn ensure_space(&mut self, height: f32) {
        if self.y - height < self.bottom() && !self.at_page_top() {
            self.new_page();
        }
    }

    /// Text als Hex-String aus Glyph-IDs; merkt sich die Glyphen für Schrift
    /// und `ToUnicode`.
    fn encode(&mut self, face: Face, text: &str) -> String {
        let used = &mut self.used[face.index()];
        let mut out = String::from("<");
        for glyph in face.font().glyphs(text) {
            used.entry(glyph.id).or_insert(glyph.text);
            let _ = write!(out, "{:04X}", glyph.id);
        }
        out.push('>');
        out
    }

    /// Text mit Grundlinie `baseline` in die Seite `page` (oder die aktuelle).
    #[allow(clippy::too_many_arguments)]
    fn show_text(
        &mut self,
        page: Option<usize>,
        face: Face,
        size: f32,
        color: Rgb,
        x: f32,
        baseline: f32,
        text: &str,
    ) {
        let glyphs = self.encode(face, text);
        let target = match page {
            Some(index) => &mut self.pages[index],
            None => &mut self.current,
        };
        let _ = writeln!(
            target,
            "{} BT /{} {size:.1} Tf {x:.2} {baseline:.2} Td {glyphs} Tj ET",
            color.fill(),
            face.resource(),
        );
    }

    fn text_line(&mut self, x: f32, face: Face, size: f32, color: Rgb, text: &str) {
        let baseline = self.y - size;
        self.show_text(None, face, size, color, x, baseline, text);
    }

    /// Umbrochener Absatz; hält jede Zeile auf einer Seite.
    fn paragraph(&mut self, x: f32, width: f32, face: Face, size: f32, color: Rgb, text: &str) {
        let leading = size * 1.3;
        for line in wrap(text, face, size, width) {
            self.ensure_space(leading);
            self.text_line(x, face, size, color, &line);
            self.y -= leading;
        }
    }

    fn gap(&mut self, height: f32) {
        self.y -= height;
    }

    /// Linie im Akzent unter dem Titel.
    fn accent_rule(&mut self, x: f32, width: f32) {
        let y = self.y;
        let _ = writeln!(
            self.current,
            "q {} 1.5 w {x:.2} {y:.2} m {:.2} {y:.2} l S Q",
            self.colors.accent.stroke(),
            x + width,
        );
    }

    /// Nummernkreis im Akzent, mittig zur ersten Textzeile mit `size`.
    fn badge(&mut self, x: f32, number: usize, size: f32) {
        let text_cap = Face::Regular.font().cap_height_at(size);
        let (radius, cx) = (BADGE / 2.0, x + BADGE / 2.0);
        let cy = self.y - size + text_cap / 2.0;
        let _ = writeln!(
            self.current,
            "{} {}",
            self.colors.accent.fill(),
            circle(cx, cy, radius)
        );
        let label = number.to_string();
        let label_size = 9.5;
        let width = Face::Bold.font().width(&label, label_size);
        let baseline = cy - Face::Bold.font().cap_height_at(label_size) / 2.0;
        let on_accent = self.colors.on_accent;
        self.show_text(
            None,
            Face::Bold,
            label_size,
            on_accent,
            cx - width / 2.0,
            baseline,
            &label,
        );
    }

    /// Schließt die letzte Seite ab und setzt auf jede Seite die Seitenzahl
    /// rechts und, falls gewünscht, den Hinweis links.
    fn finish(&mut self, credit: Option<&str>) {
        self.pages.push(std::mem::take(&mut self.current));
        let total = self.pages.len();
        let geometry = self.geometry;
        let size = 9.0;
        let baseline = (geometry.margin - size).max(size);
        let muted = self.colors.muted;
        for index in 0..total {
            let label = format!("{} / {total}", index + 1);
            let x = geometry.width - geometry.margin - Face::Regular.font().width(&label, size);
            self.show_text(Some(index), Face::Regular, size, muted, x, baseline, &label);
            if let Some(credit) = credit {
                self.show_text(
                    Some(index),
                    Face::Regular,
                    size,
                    muted,
                    geometry.margin,
                    baseline,
                    credit,
                );
            }
        }
    }
}

/// Gefüllter Kreis aus vier Bézier-Bögen.
fn circle(cx: f32, cy: f32, r: f32) -> String {
    let k = 0.552_284_8 * r;
    format!(
        "{:.2} {cy:.2} m \
         {:.2} {:.2} {:.2} {:.2} {cx:.2} {:.2} c \
         {:.2} {:.2} {:.2} {:.2} {:.2} {cy:.2} c \
         {:.2} {:.2} {:.2} {:.2} {cx:.2} {:.2} c \
         {:.2} {:.2} {:.2} {:.2} {:.2} {cy:.2} c f",
        cx + r,
        cx + r,
        cy + k,
        cx + k,
        cy + r,
        cy + r,
        cx - k,
        cy + r,
        cx - r,
        cy + k,
        cx - r,
        cx - r,
        cy - k,
        cx - k,
        cy - r,
        cy - r,
        cx + k,
        cy - r,
        cx + r,
        cy - k,
        cx + r,
    )
}

/// Rendert den Guide als PDF (eine Spalte, ein Screenshot je Schritt).
pub fn pdf(file: &StepsFile, mode: ExportMode, page: PdfPage, style: &ExportStyle) -> Vec<u8> {
    let guide = &file.guide;
    let lang = language(guide);
    let title = title_or_default(guide, lang);
    let geometry = Geometry::new(page);
    let (left, content_width) = (geometry.margin, geometry.content_width());
    let mut layout = Layout::new(geometry, Colors::new(&style.palette(Skin::Light)));
    let (ink, muted) = (layout.colors.ink, layout.colors.muted);

    layout.paragraph(left, content_width, Face::Bold, 20.0, ink, &title);
    layout.gap(2.0);
    layout.accent_rule(left, content_width);
    layout.gap(12.0);
    if let Some(description) = &guide.description {
        layout.paragraph(left, content_width, Face::Regular, 11.0, muted, description);
        layout.gap(6.0);
    }
    if mode == ExportMode::BugReport {
        for line in guide_metadata_lines(file, lang) {
            layout.paragraph(left, content_width, Face::Regular, 9.0, muted, &line);
        }
        layout.gap(6.0);
    }
    layout.gap(8.0);

    for (number, step) in guide.steps.iter().enumerate() {
        let text_width = content_width - STEP_INDENT;
        let lines = wrap(step.text.effective(), Face::Regular, 12.0, text_width);
        let leading = 12.0 * 1.3;
        let image = file
            .images
            .get(&step.id)
            .and_then(|png| decode_image(&export_png(step, png)));
        let image_size = image
            .as_ref()
            .map(|image| fit_image(image, text_width, geometry.height * MAX_IMAGE_SHARE));

        // Nummer, erste Textzeile und Screenshot gehören zusammen.
        let head = leading * f32::from(u16::try_from(lines.len().max(1)).unwrap_or(u16::MAX));
        let block = head + image_size.map_or(0.0, |(_, height)| height + 8.0);
        layout.ensure_space(block.min(layout.usable_height()).max(BADGE));

        layout.badge(left, number + 1, 12.0);
        if lines.is_empty() {
            layout.gap(leading.max(BADGE));
        }
        for line in &lines {
            layout.ensure_space(leading);
            layout.text_line(left + STEP_INDENT, Face::Regular, 12.0, ink, line);
            layout.gap(leading);
        }

        if let (Some(image), Some((width, height))) = (image, image_size) {
            layout.gap(4.0);
            layout.ensure_space(height);
            let index = layout.images.len();
            let x = left + STEP_INDENT;
            let y = layout.y - height;
            let _ = writeln!(
                layout.current,
                "q {width:.2} 0 0 {height:.2} {x:.2} {y:.2} cm /Im{index} Do Q\n\
                 q {} 0.75 w {x:.2} {y:.2} {width:.2} {height:.2} re S Q",
                layout.colors.line.stroke(),
            );
            layout.images.push(image);
            layout.gap(height + 4.0);
        }

        if mode == ExportMode::BugReport {
            for line in step_metadata_lines(step, lang) {
                layout.paragraph(
                    left + STEP_INDENT,
                    text_width,
                    Face::Regular,
                    8.5,
                    muted,
                    &line,
                );
            }
        }
        layout.gap(14.0);
    }
    layout.finish(style.credit_text(lang));

    write_document(&title, layout)
}

/// Schreibt den PDF-Export als Datei.
pub fn write_pdf_export(
    file: &StepsFile,
    mode: ExportMode,
    page: PdfPage,
    style: &ExportStyle,
    path: impl AsRef<std::path::Path>,
) -> std::io::Result<()> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(path, pdf(file, mode, page, style))
}

fn guide_metadata_lines(file: &StepsFile, lang: Locale) -> Vec<String> {
    let guide = &file.guide;
    let mut lines = vec![labeled(
        lang,
        Key::ExportCreated,
        &format_utc(guide.created_at_ms),
    )];
    if let Some(environment) = &guide.environment {
        let parts: Vec<&str> = [&environment.os, &environment.os_version]
            .into_iter()
            .filter_map(|part| part.as_deref())
            .collect();
        if !parts.is_empty() {
            lines.push(labeled(lang, Key::ExportEnvironment, &parts.join(" ")));
        }
    }
    lines
}

/// Pixel in Punkt bei 96 dpi, wie der Browser das Bild im HTML-Export zeigt.
const POINTS_PER_PIXEL: f32 = 0.75;

/// Verkleinert auf die Textbreite und `max_height`, vergrößert aber nie über
/// die natürliche Größe (wie `max-width: 100%` im HTML-Export).
fn fit_image(image: &PdfImage, max_width: f32, max_height: f32) -> (f32, f32) {
    #[allow(clippy::cast_precision_loss)]
    let (pixel_width, pixel_height) = (image.width.max(1) as f32, image.height as f32);
    let aspect = pixel_height / pixel_width;
    let width = max_width.min(pixel_width * POINTS_PER_PIXEL);
    let height = width * aspect;
    if height > max_height {
        (max_height / aspect, max_height)
    } else {
        (width, height)
    }
}

/// PNG nach RGB ohne Alpha, weil Screenshots deckend sind, und dann nach zlib.
fn decode_image(png: &[u8]) -> Option<PdfImage> {
    let rgb = image::load_from_memory(png).ok()?.to_rgb8();
    let (width, height) = rgb.dimensions();
    if width == 0 || height == 0 {
        return None;
    }
    Some(PdfImage {
        width,
        height,
        data: miniz_oxide::deflate::compress_to_vec_zlib(rgb.as_raw(), 6),
    })
}

/// Objekte einer eingebetteten Schrift, in dieser Reihenfolge nummeriert.
const FONT_OBJECTS: usize = 5;

fn write_document(title: &str, layout: Layout) -> Vec<u8> {
    let Layout {
        geometry,
        pages,
        images,
        used,
        ..
    } = layout;
    let (page_width, page_height) = (geometry.width, geometry.height);
    let page_count = pages.len();

    // Objekt-Nummern: 1 Katalog, 2 Seitenbaum, 3 Ressourcen, 4 Info, dann je
    // Schnitt Type0, CIDFont, Deskriptor, Schriftdatei, ToUnicode, dann
    // Bilder, dann je Seite Inhalt + Seite.
    let first_font = 5;
    let font_object = |face: Face| first_font + face.index() * FONT_OBJECTS;
    let first_image = first_font + Face::ALL.len() * FONT_OBJECTS;
    let first_page = first_image + images.len();
    let page_object = |page: usize| first_page + page * 2 + 1;

    let mut out: Vec<u8> = Vec::new();
    let mut offsets: Vec<usize> = Vec::new();
    out.extend_from_slice(b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n");

    let mut object = |out: &mut Vec<u8>, body: &[u8]| {
        offsets.push(out.len());
        let _ = writeln!(ByteWriter(out), "{} 0 obj", offsets.len());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    };

    object(&mut out, b"<< /Type /Catalog /Pages 2 0 R >>");
    let kids: Vec<String> = (0..page_count)
        .map(|page| format!("{} 0 R", page_object(page)))
        .collect();
    object(
        &mut out,
        format!(
            "<< /Type /Pages /Kids [{}] /Count {page_count} >>",
            kids.join(" ")
        )
        .as_bytes(),
    );
    let mut xobjects = String::new();
    for index in 0..images.len() {
        let _ = write!(xobjects, "/Im{index} {} 0 R ", first_image + index);
    }
    object(
        &mut out,
        format!(
            "<< /Font << /F1 {} 0 R /F2 {} 0 R >> /XObject << {xobjects}>> >>",
            font_object(Face::Regular),
            font_object(Face::Bold)
        )
        .as_bytes(),
    );
    object(
        &mut out,
        format!(
            "<< /Title {} /Producer (steps-export) >>",
            utf16_hex_string(title)
        )
        .as_bytes(),
    );

    for face in Face::ALL {
        for body in font_objects(face, font_object(face), &used[face.index()]) {
            object(&mut out, &body);
        }
    }

    for image in &images {
        object(&mut out, &image_object(image));
    }

    for (index, content) in pages.iter().enumerate() {
        let mut body = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
        body.extend_from_slice(content.as_bytes());
        body.extend_from_slice(b"endstream");
        object(&mut out, &body);
        object(
            &mut out,
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {page_width} {page_height}] \
                 /Resources 3 0 R /Contents {} 0 R >>",
                page_object(index) - 1
            )
            .as_bytes(),
        );
    }

    let xref_offset = out.len();
    let size = offsets.len() + 1;
    let _ = write!(
        ByteWriter(&mut out),
        "xref\n0 {size}\n0000000000 65535 f \n"
    );
    for offset in &offsets {
        let _ = writeln!(ByteWriter(&mut out), "{offset:010} 00000 n ");
    }
    let _ = write!(
        ByteWriter(&mut out),
        "trailer\n<< /Size {size} /Root 1 0 R /Info 4 0 R >>\nstartxref\n{xref_offset}\n%%EOF\n"
    );
    out
}

/// Die fünf Objekte einer eingebetteten Schrift ab Nummer `first`: Type0,
/// `CIDFontType2`, Deskriptor, Schriftdatei (Teilmenge) und `ToUnicode`.
fn font_objects(face: Face, first: usize, used: &BTreeMap<u16, char>) -> Vec<Vec<u8>> {
    let font = face.font();
    let ids: BTreeSet<u16> = used.keys().copied().collect();
    let name = format!("{}+{}", subset_tag(face, &ids), font.postscript_name());
    let (cid, descriptor, file, to_unicode) = (first + 1, first + 2, first + 3, first + 4);

    let mut widths = String::new();
    for id in &ids {
        let _ = write!(widths, "{id} [{}] ", font.advance(*id));
    }
    let [x_min, y_min, x_max, y_max] = font.bounding_box();
    let program = font.subset(&ids);
    let compressed = miniz_oxide::deflate::compress_to_vec_zlib(&program, 9);
    let mut file_body = format!(
        "<< /Length {} /Length1 {} /Filter /FlateDecode >>\nstream\n",
        compressed.len(),
        program.len()
    )
    .into_bytes();
    file_body.extend_from_slice(&compressed);
    file_body.extend_from_slice(b"\nendstream");

    let cmap = to_unicode_cmap(used);
    let mut cmap_body = format!("<< /Length {} >>\nstream\n", cmap.len()).into_bytes();
    cmap_body.extend_from_slice(cmap.as_bytes());
    cmap_body.extend_from_slice(b"endstream");

    vec![
        format!(
            "<< /Type /Font /Subtype /Type0 /BaseFont /{name} /Encoding /Identity-H \
             /DescendantFonts [{cid} 0 R] /ToUnicode {to_unicode} 0 R >>"
        )
        .into_bytes(),
        format!(
            "<< /Type /Font /Subtype /CIDFontType2 /BaseFont /{name} \
             /CIDSystemInfo << /Registry (Adobe) /Ordering (Identity) /Supplement 0 >> \
             /FontDescriptor {descriptor} 0 R /CIDToGIDMap /Identity /W [{widths}] >>"
        )
        .into_bytes(),
        format!(
            "<< /Type /FontDescriptor /FontName /{name} /Flags 32 \
             /FontBBox [{x_min} {y_min} {x_max} {y_max}] /ItalicAngle {} /Ascent {} \
             /Descent {} /CapHeight {} /StemV {} /FontFile2 {file} 0 R >>",
            font.italic_angle(),
            font.ascent(),
            font.descent(),
            font.cap_height(),
            face.stem(),
        )
        .into_bytes(),
        file_body,
        cmap_body,
    ]
}

/// Sechs Großbuchstaben, die eine Teilmenge kennzeichnen (PDF 1.7, 9.6.4).
/// Gleiche Glyphen ergeben dasselbe Präfix.
fn subset_tag(face: Face, ids: &BTreeSet<u16>) -> String {
    let mut hash: u32 = 0x811C_9DC5 ^ u32::try_from(face.index()).unwrap_or(0);
    for id in ids {
        for byte in id.to_be_bytes() {
            hash = (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193);
        }
    }
    (0..6)
        .map(|_| {
            let letter = b'A' + u8::try_from(hash % 26).unwrap_or(0);
            hash /= 26;
            char::from(letter)
        })
        .collect()
}

fn image_object(image: &PdfImage) -> Vec<u8> {
    let mut body = format!(
        "<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB \
         /BitsPerComponent 8 /Filter /FlateDecode /Length {} >>\nstream\n",
        image.width,
        image.height,
        image.data.len()
    )
    .into_bytes();
    body.extend_from_slice(&image.data);
    body.extend_from_slice(b"\nendstream");
    body
}

/// `ToUnicode`-Tabelle: jede benutzte Glyphen-ID auf ihr Zeichen (UTF-16BE).
fn to_unicode_cmap(used: &BTreeMap<u16, char>) -> String {
    let mut out = String::from(
        "/CIDInit /ProcSet findresource begin\n12 dict begin\nbegincmap\n\
         /CIDSystemInfo << /Registry (Adobe) /Ordering (UCS) /Supplement 0 >> def\n\
         /CMapName /Adobe-Identity-UCS def\n/CMapType 2 def\n\
         1 begincodespacerange\n<0000> <FFFF>\nendcodespacerange\n",
    );
    let entries: Vec<(&u16, &char)> = used.iter().collect();
    for chunk in entries.chunks(100) {
        let _ = writeln!(out, "{} beginbfchar", chunk.len());
        for (id, c) in chunk {
            let _ = write!(out, "<{id:04X}> <");
            for unit in c.encode_utf16(&mut [0u16; 2]) {
                let _ = write!(out, "{unit:04X}");
            }
            out.push_str(">\n");
        }
        let _ = writeln!(out, "endbfchar");
    }
    out.push_str("endcmap\nCMapName currentdict /CMap defineresource pop\nend\nend\n");
    out
}

/// `fmt::Write` auf einen Byte-Puffer (PDF-Strukturen sind ASCII).
struct ByteWriter<'a>(&'a mut Vec<u8>);

impl std::fmt::Write for ByteWriter<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.0.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

/// Info-Strings dürfen Unicode sein: UTF-16BE mit BOM.
fn utf16_hex_string(text: &str) -> String {
    let mut out = String::from("<FEFF");
    for unit in text.encode_utf16() {
        let _ = write!(out, "{unit:04X}");
    }
    out.push('>');
    out
}

fn text_width(text: &str, face: Face, size: f32) -> f32 {
    face.font().width(text, size)
}

/// Wortweiser Umbruch; überlange Wörter werden zeichenweise geteilt.
fn wrap(text: &str, face: Face, size: f32, width: f32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() {
            word.to_owned()
        } else {
            format!("{line} {word}")
        };
        if text_width(&candidate, face, size) <= width {
            line = candidate;
            continue;
        }
        if !line.is_empty() {
            lines.push(std::mem::take(&mut line));
        }
        if text_width(word, face, size) <= width {
            word.clone_into(&mut line);
            continue;
        }
        for c in word.chars() {
            line.push(c);
            if text_width(&line, face, size) > width {
                line.pop();
                lines.push(std::mem::take(&mut line));
                line.push(c);
            }
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_writes_glyph_ids_and_remembers_the_characters() {
        let geometry = Geometry::new(PdfPage::default());
        let mut layout = Layout::new(
            geometry,
            Colors::new(&ExportStyle::default().palette(Skin::Light)),
        );
        let hex = layout.encode(Face::Regular, "Łó");
        assert_eq!(hex.len(), 2 + 8, "{hex}");
        let chars: Vec<char> = layout.used[0].values().copied().collect();
        assert!(chars.contains(&'Ł') && chars.contains(&'ó'), "{chars:?}");
        assert!(layout.used[1].is_empty());
    }

    #[test]
    fn to_unicode_maps_ids_to_utf16() {
        let cmap = to_unicode_cmap(&BTreeMap::from([(7, 'Ł'), (9, '😀')]));
        assert!(cmap.contains("2 beginbfchar"));
        assert!(cmap.contains("<0007> <0141>"));
        assert!(cmap.contains("<0009> <D83DDE00>"));
    }

    #[test]
    fn subset_tag_is_six_letters_and_stable() {
        let ids = BTreeSet::from([3, 4, 5]);
        let tag = subset_tag(Face::Regular, &ids);
        assert_eq!(tag.len(), 6);
        assert!(tag.chars().all(|c| c.is_ascii_uppercase()));
        assert_eq!(tag, subset_tag(Face::Regular, &ids));
        assert_ne!(tag, subset_tag(Face::Bold, &ids));
    }

    #[test]
    fn wrap_respects_width_and_splits_long_words() {
        let lines = wrap("Klick auf Speichern im Dialog", Face::Regular, 12.0, 80.0);
        assert!(lines.len() > 1);
        for line in &lines {
            assert!(text_width(line, Face::Regular, 12.0) <= 80.0, "{line}");
        }
        let long = wrap(&"x".repeat(200), Face::Regular, 12.0, 100.0);
        assert!(long.len() > 1);
        assert_eq!(long.concat().len(), 200);
    }

    fn image(width: u32, height: u32) -> PdfImage {
        PdfImage {
            width,
            height,
            data: Vec::new(),
        }
    }

    #[test]
    fn fit_image_caps_tall_screenshots() {
        let (width, height) = fit_image(&image(1000, 10_000), 500.0, 400.0);
        assert!(height <= 400.01);
        assert!(width < 500.0);
    }

    #[test]
    fn fit_image_shrinks_wide_screenshots_but_never_upscales() {
        let (width, _) = fit_image(&image(1920, 1080), 500.0, 800.0);
        assert!((width - 500.0).abs() < 0.01);
        let (width, height) = fit_image(&image(1, 1), 500.0, 800.0);
        assert!(width <= 1.0 && height <= 1.0);
    }

    #[test]
    fn geometry_follows_paper_and_margin() {
        let a4 = Geometry::new(PdfPage::default());
        assert!((a4.width - 595.28).abs() < 0.01);
        assert!((a4.margin - 18.0 * POINTS_PER_MM).abs() < 0.01);

        let letter = Geometry::new(PdfPage {
            paper: Paper::Letter,
            margin_mm: 20,
        });
        assert!((letter.height - 792.0).abs() < 0.01);
        assert!((letter.margin - 20.0 * POINTS_PER_MM).abs() < 0.01);
        assert!((letter.content_width() - (612.0 - 40.0 * POINTS_PER_MM)).abs() < 0.01);

        let huge = Geometry::new(PdfPage {
            paper: Paper::A4,
            margin_mm: 500,
        });
        assert!(huge.content_width() >= huge.width / 2.0 - 0.01);
    }

    #[test]
    fn paper_codes_round_trip() {
        for paper in Paper::ALL {
            assert_eq!(Paper::from_code(paper.code()), Some(paper));
        }
        assert_eq!(Paper::from_code(" Letter "), Some(Paper::Letter));
        assert_eq!(Paper::from_code("a5"), None);
    }
}
