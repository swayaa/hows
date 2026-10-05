//! Tests der Markdown-, HTML-, PDF- und JSON-Exporte in beiden Output-Modi.

use steps_export::brand::{self, Skin};
use steps_export::{
    html, json, markdown, markdown_images, pdf, thumbnail_data_uri, write_html_export,
    write_markdown_export, write_pdf_export, ExportMode, ExportStyle, Paper, PdfPage,
};
use steps_i18n::{Key, Locale};
use steps_store::{
    marks, Action, ElementInfo, Environment, Guide, Monitor, Overlay, Position, ScrollDirection,
    Step, StepText, StepsFile,
};

/// Minimales gültiges 1×1-PNG (nur für Exporte ohne Burn-in).
const PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 1, 2, 3];

/// Sonderzeichen aller UI-Sprachen plus Zeichen außerhalb von `WinAnsiEncoding`.
const SONDERZEICHEN: [&str; 8] = [
    "Größe ändern: Äpfel, Öl, Übung, Straße",
    "Créer « Dossier » à côté : œuvre, ça, où",
    "¿Añadir cuña? ¡Sí!",
    "Configuração não disponível à direita",
    "Perché è così? Più città",
    "Één überprüfen, reünie",
    "Zażółć gęślą jaźń, Łódź",
    "Weiter → Fertig",
];

fn style() -> ExportStyle {
    ExportStyle::default()
}

fn ohne_credit() -> ExportStyle {
    ExportStyle {
        credit: false,
        ..ExportStyle::default()
    }
}

fn credit(language: &str) -> &'static str {
    Locale::resolve(language).text(Key::ExportCredit)
}

fn sonderzeichen_datei() -> StepsFile {
    let mut file = beispiel_datei("pl");
    let template = file.guide.steps[1].clone();
    "Übersicht → Łódź".clone_into(&mut file.guide.title);
    file.guide.steps = SONDERZEICHEN
        .iter()
        .enumerate()
        .map(|(index, text)| Step {
            id: format!("step-{index:04}"),
            text: StepText::generated(*text),
            ..template.clone()
        })
        .collect();
    file
}

fn solid_rgba_png(width: u32, height: u32) -> Vec<u8> {
    use image::{ImageBuffer, Rgba};
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(width, height, Rgba([255, 255, 255, 255]));
    let mut out = std::io::Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut out, image::ImageFormat::Png)
        .expect("png");
    out.into_inner()
}

fn beispiel_datei(language: &str) -> StepsFile {
    // 2026-09-02 00:00 UTC
    let mut guide = Guide::new("Netzlaufwerk verbinden", 1_788_307_200_000, language);
    guide.description = Some("Kurzanleitung für neue Kolleginnen und Kollegen.".to_owned());
    guide.environment = Some(Environment {
        os: Some("Windows".to_owned()),
        os_version: Some("10.0.26100".to_owned()),
    });
    guide.steps = vec![
        Step {
            id: "step-0001".to_owned(),
            action: Action::Click,
            timestamp_ms: 1_788_307_260_000,
            position: Some(Position { x: 100, y: 200 }),
            monitor: Some(Monitor {
                index: 0,
                x: 0,
                y: 0,
                width: 2560,
                height: 1440,
                scale_factor: 1.5,
            }),
            element: Some(ElementInfo {
                name: Some("Speichern".to_owned()),
                control_type: Some("Button".to_owned()),
                automation_id: Some("btnSave".to_owned()),
                window_title: Some("Explorer <intern> & Co".to_owned()),
                app_name: Some("explorer".to_owned()),
                ..ElementInfo::default()
            }),
            text: StepText::generated("Klick auf „Speichern“"),
            overlays: Vec::new(),
            crop: None,
        },
        Step {
            id: "step-0002".to_owned(),
            action: Action::Scroll {
                direction: ScrollDirection::Down,
                amount: 3,
            },
            timestamp_ms: 1_788_307_320_000,
            position: None,
            monitor: None,
            element: None,
            text: StepText {
                generated: "Nach unten gescrollt".to_owned(),
                override_text: Some("Zur Freigabe „Projekte“ scrollen".to_owned()),
            },
            overlays: Vec::new(),
            crop: None,
        },
    ];

    let mut file = StepsFile::new(guide);
    file.images.insert("step-0001".to_owned(), PNG.to_vec());
    file
}

// --- Markdown -----------------------------------------------------------------

#[test]
fn markdown_sop_enthaelt_schritte_und_bildreferenzen() {
    let file = beispiel_datei("de");
    let md = markdown(&file, ExportMode::Sop, &style());

    assert!(md.starts_with("# Netzlaufwerk verbinden\n"));
    assert!(md.contains("1. Klick auf „Speichern“"));
    assert!(md.contains("![Schritt 1](images/step-0001.png)"));
    // Override hat Vorrang:
    assert!(md.contains("2. Zur Freigabe „Projekte“ scrollen"));
    // Schritt ohne Bild referenziert keines:
    assert!(!md.contains("step-0002.png"));
    // SOP blendet technische Details aus:
    assert!(!md.contains("10.0.26100"));
    assert!(!md.contains("AutomationId"));
    assert!(!md.contains("Erstellt:"));
}

#[test]
fn markdown_credit_folgt_der_einstellung() {
    let file = beispiel_datei("de");
    let with = markdown(&file, ExportMode::Sop, &style());
    assert!(
        with.ends_with(&format!("---\n\n*{}*\n", credit("de"))),
        "{with}"
    );
    let without = markdown(&file, ExportMode::Sop, &ohne_credit());
    assert!(!without.contains(credit("de")));
    assert!(!without.contains("Hows"));
}

#[test]
fn markdown_bug_report_enthaelt_technische_metadaten() {
    let file = beispiel_datei("de");
    let md = markdown(&file, ExportMode::BugReport, &style());

    assert!(md.contains("Erstellt: 2026-09-02 00:00 UTC"));
    assert!(md.contains("Umgebung: Windows 10.0.26100"));
    assert!(md.contains("Element: Speichern, Button, AutomationId=btnSave"));
    assert!(md.contains("App: explorer"));
    assert!(md.contains("Position: (100, 200)"));
    assert!(md.contains("Bildschirm: 0 (2560×1440, Skalierung: 150%)"));
}

#[test]
fn markdown_images_liefert_nur_referenzierte_bilder() {
    let file = beispiel_datei("de");
    let images = markdown_images(&file);
    assert_eq!(images.len(), 1);
    assert_eq!(images["images/step-0001.png"], PNG);
}

#[test]
fn html_brannt_overlays_ein_original_bleibt_sauber() {
    let mut file = beispiel_datei("de");
    let original = solid_rgba_png(48, 48);
    file.images.insert("step-0001".to_owned(), original.clone());
    file.guide.steps[0].overlays = vec![Overlay::Rect {
        id: "o1".to_owned(),
        color: marks().color.clone(),
        stroke: marks().stroke,
        x: 0.2,
        y: 0.2,
        w: 0.5,
        h: 0.4,
    }];

    let html_out = html(&file, ExportMode::Sop, &style());
    assert!(html_out.contains("data:image/png;base64,"));
    // HTML darf nicht mehr die Original-Base64 der weißen Fläche allein tragen,
    // wenn Overlays eingebrannt wurden. Die Export-PNG ist nicht das Original.
    let md_images = markdown_images(&file);
    let burned = &md_images["images/step-0001.png"];
    assert_ne!(burned.as_slice(), original.as_slice());
    // Quelle in StepsFile bleibt unmarkiert:
    assert_eq!(file.images["step-0001"], original);
}

#[test]
fn write_markdown_export_schreibt_dokument_und_bilder() {
    let dir = std::env::temp_dir().join(format!("steps-export-test-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();

    let file = beispiel_datei("de");
    write_markdown_export(&file, ExportMode::Sop, &style(), &dir).expect("Export");

    let md = std::fs::read_to_string(dir.join("guide.md")).expect("guide.md");
    assert!(md.contains("Netzlaufwerk verbinden"));
    let png = std::fs::read(dir.join("images/step-0001.png")).expect("Bild");
    assert_eq!(png, PNG);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn markdown_export_schreibt_kein_bild_ausserhalb_des_ordners() {
    let root = std::env::temp_dir().join(format!("steps-export-escape-{}", std::process::id()));
    std::fs::remove_dir_all(&root).ok();
    let dir = root.join("guides").join("export");
    let victim = root.join("guides").join("victim.png");

    for id in [r"..\..\victim", "../../victim"] {
        let mut file = beispiel_datei("de");
        file.guide.steps[0].id = id.to_owned();
        file.images.clear();
        file.images.insert(id.to_owned(), PNG.to_vec());

        let written = write_markdown_export(&file, ExportMode::Sop, &style(), &dir);

        let target = dir.join(format!("images/{id}.png"));
        assert!(
            !victim.exists(),
            "{id}: Bild landete in {}",
            victim.display()
        );
        if written.is_ok() {
            let inside = std::fs::canonicalize(&target)
                .expect("geschriebenes Bild")
                .starts_with(std::fs::canonicalize(&dir).expect("Exportordner"));
            assert!(inside, "{id}: {} liegt außerhalb", target.display());
        }
        std::fs::remove_dir_all(&dir).ok();
    }

    std::fs::remove_dir_all(&root).ok();
}

#[test]
fn markdown_englische_beschriftungen() {
    let file = beispiel_datei("en");
    let md = markdown(&file, ExportMode::BugReport, &style());
    assert!(md.contains("Created: 2026-09-02 00:00 UTC"));
    assert!(md.contains("![Step 1](images/step-0001.png)"));
    assert!(md.contains("Scale: 150%"));
}

#[test]
fn franzoesische_beschriftungen_mit_schmalem_leerzeichen() {
    let file = beispiel_datei("fr");
    let md = markdown(&file, ExportMode::BugReport, &style());
    assert!(
        md.contains("Date de création\u{a0}: 2026-09-02 00:00 UTC"),
        "{md}"
    );
    assert!(md.contains("Écran\u{a0}: 0 (2560×1440, Mise à l'échelle\u{a0}: 150%)"));
    assert!(!md.contains("Application: "));
    let html_out = html(&file, ExportMode::BugReport, &style());
    assert!(html_out.contains("Environnement\u{a0}: Windows 10.0.26100"));
}

// --- HTML ----------------------------------------------------------------------

#[test]
fn html_ist_selbst_enthalten_mit_inline_bildern() {
    let file = beispiel_datei("de");
    let html = html(&file, ExportMode::Sop, &style());

    assert!(html.contains("<!doctype html>"));
    assert!(html.contains("data:image/png;base64,iVBORw0KGgoBAgM="));
    // Keine externen Referenzen (der moderne MHT-Ersatz, Spec §6):
    assert!(!html.contains("http://"));
    assert!(!html.contains("https://"));
    assert!(!html.contains("src=\"images/"));
}

#[test]
fn html_traegt_markenfarben_und_eingebettete_schrift() {
    let html = html(&beispiel_datei("de"), ExportMode::Sop, &style());
    let light = brand::brand_palette(brand::default_brand(), Skin::Light, "");
    let dark = brand::brand_palette(brand::default_brand(), Skin::Dark, "");
    for (token, hex) in light.tokens() {
        let variable = format!("--sl-{}:{hex};", token.replace('_', "-"));
        assert!(html.contains(&variable), "{variable}");
    }
    let dark_block = html
        .split("prefers-color-scheme:dark")
        .nth(1)
        .expect("dunkles Thema");
    assert!(dark_block.contains(&format!("--sl-accent:{};", dark.accent)));

    let font = brand::font();
    assert!(html.contains(&format!("font-family:\"{}\"", font.family)));
    assert!(html.contains(&format!("--sl-font:{}", font.css_stack())));
    assert_eq!(html.matches("src:url(data:font/woff2;base64,").count(), 2);
    assert!(html.contains("unicode-range:"));
}

/// Pixelwerte hinter `marker` bis zum nächsten Zeichen, das keine Ziffer ist.
fn px_after<'a>(text: &'a str, marker: &'a str) -> impl Iterator<Item = u32> + 'a {
    text.split(marker).skip(1).filter_map(|rest| {
        let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
        rest[digits.len()..]
            .starts_with("px")
            .then(|| digits.parse().ok())
            .flatten()
    })
}

#[test]
fn html_radien_liegen_auf_der_skala_der_app() {
    let tokens = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../app/src/styles/tokens.css"),
    )
    .expect("tokens.css");
    let scale: Vec<u32> = tokens
        .lines()
        .filter(|line| line.trim_start().starts_with("--sl-radius"))
        .flat_map(|line| px_after(line, ": ").collect::<Vec<_>>())
        .collect();
    assert!(scale.len() >= 3, "Radiusskala in tokens.css: {scale:?}");

    let html = html(&beispiel_datei("de"), ExportMode::Sop, &style());
    let used: Vec<u32> = px_after(&html, "border-radius:").collect();
    assert_ne!(used.len(), 0);
    for radius in used {
        assert!(scale.contains(&radius), "{radius}px fehlt in {scale:?}");
    }
}

#[test]
fn html_nutzt_marke_und_eigenen_akzent() {
    let custom = ExportStyle {
        brand: "ink".to_owned(),
        accent: "#AA3366".to_owned(),
        credit: true,
    };
    let html = html(&beispiel_datei("de"), ExportMode::Sop, &custom);
    for skin in [Skin::Light, Skin::Dark] {
        let palette = brand::brand_palette("ink", skin, "#AA3366");
        assert!(html.contains(&format!("--sl-accent:{};", palette.accent)));
        assert!(html.contains(&format!("--sl-bg:{};", palette.bg)));
    }
    let preset = brand::brand_palette("ink", Skin::Light, "");
    assert_ne!(preset.accent, "#AA3366");
}

#[test]
fn html_credit_folgt_der_einstellung() {
    let file = beispiel_datei("fr");
    let with = html(&file, ExportMode::Sop, &style());
    assert!(with.contains(&format!(
        "<footer class=\"credit\">{}</footer>",
        credit("fr")
    )));
    let without = html(&file, ExportMode::Sop, &ohne_credit());
    assert!(!without.contains("class=\"credit\""));
    assert!(!without.contains(credit("fr")));
}

#[test]
fn html_behaelt_sonderzeichen() {
    let html = html(&sonderzeichen_datei(), ExportMode::Sop, &style());
    for text in SONDERZEICHEN {
        assert!(html.contains(text), "{text}");
    }
}

#[test]
fn html_escaped_nutzertexte() {
    let file = beispiel_datei("de");
    let html = html(&file, ExportMode::BugReport, &style());
    // Fenstertitel „Explorer <intern> & Co“ muss escaped sein:
    assert!(html.contains("Explorer &lt;intern&gt; &amp; Co"));
    assert!(!html.contains("<intern>"));
}

#[test]
fn html_sop_ohne_metadaten_bug_report_mit() {
    let file = beispiel_datei("de");
    let sop = html(&file, ExportMode::Sop, &style());
    let bug = html(&file, ExportMode::BugReport, &style());

    assert!(!sop.contains("10.0.26100"));
    assert!(bug.contains("10.0.26100"));
    assert!(bug.contains("Zeit: 2026-09-02 00:01 UTC"));
}

#[test]
fn write_html_export_schreibt_selbst_enthaltene_datei() {
    let path = std::env::temp_dir().join(format!(
        "steps-html-export-test-{}.html",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);

    let file = beispiel_datei("de");
    write_html_export(&file, ExportMode::Sop, &style(), &path).expect("Export");

    let html = std::fs::read_to_string(&path).expect("HTML");
    assert!(html.contains("<!doctype html>"));
    assert!(html.contains("Netzlaufwerk verbinden"));
    assert!(html.contains("data:image/png;base64,"));

    let _ = std::fs::remove_file(&path);
}

// --- JSON ----------------------------------------------------------------------

#[test]
fn json_sop_enthaelt_nur_anzeige_daten() {
    let file = beispiel_datei("de");
    let value: serde_json::Value = serde_json::from_str(&json(&file, ExportMode::Sop)).unwrap();

    assert_eq!(value["mode"], "sop");
    assert_eq!(value["title"], "Netzlaufwerk verbinden");
    assert_eq!(value["steps"][0]["number"], 1);
    assert_eq!(value["steps"][0]["action"], "click");
    assert_eq!(value["steps"][0]["image"], "images/step-0001.png");
    assert_eq!(
        value["steps"][1]["text"],
        "Zur Freigabe „Projekte“ scrollen"
    );
    assert_eq!(value["steps"][1]["image"], serde_json::Value::Null);
    assert!(value.get("environment").is_none());
    assert!(value["steps"][0].get("element").is_none());
}

#[test]
fn json_bug_report_enthaelt_alle_metadaten() {
    let file = beispiel_datei("de");
    let value: serde_json::Value =
        serde_json::from_str(&json(&file, ExportMode::BugReport)).unwrap();

    assert_eq!(value["mode"], "bug_report");
    assert_eq!(value["environment"]["os_version"], "10.0.26100");
    assert_eq!(value["created_at_ms"], 1_788_307_200_000_u64);
    let step = &value["steps"][0];
    assert_eq!(step["element"]["automation_id"], "btnSave");
    assert_eq!(step["position"]["x"], 100);
    assert_eq!(step["monitor"]["scale_factor"], 1.5);
    assert_eq!(step["action_detail"]["type"], "click");
    let scroll = &value["steps"][1];
    assert_eq!(scroll["action_detail"]["type"], "scroll");
    assert_eq!(scroll["action_detail"]["direction"], "down");
    assert_eq!(scroll["action_detail"]["amount"], 3);
}

#[test]
fn json_leerer_guide_ist_gueltig() {
    let file = StepsFile::new(Guide::new("", 0, "en"));
    let value: serde_json::Value = serde_json::from_str(&json(&file, ExportMode::Sop)).unwrap();
    assert_eq!(value["steps"].as_array().unwrap().len(), 0);
}

// --- PDF -----------------------------------------------------------------------

fn image_xobject_count(document: &lopdf::Document) -> usize {
    document
        .objects
        .values()
        .filter_map(|object| object.as_stream().ok())
        .filter(|stream| {
            stream
                .dict
                .get(b"Subtype")
                .and_then(lopdf::Object::as_name)
                .is_ok_and(|name| name == b"Image")
        })
        .count()
}

fn pdf_text(document: &lopdf::Document) -> String {
    let pages: Vec<u32> = document.get_pages().keys().copied().collect();
    document.extract_text(&pages).expect("Text")
}

fn dicts_of_type<'a>(
    document: &'a lopdf::Document,
    kind: &'a [u8],
) -> impl Iterator<Item = &'a lopdf::Dictionary> + 'a {
    document
        .objects
        .values()
        .filter_map(|object| object.as_dict().ok())
        .filter(move |dict| {
            dict.get(b"Type")
                .and_then(lopdf::Object::as_name)
                .is_ok_and(|name| name == kind)
        })
}

/// Name und entpackte Schriftdatei jeder eingebetteten Schrift.
fn eingebettete_schriften(document: &lopdf::Document) -> Vec<(String, Vec<u8>)> {
    dicts_of_type(document, b"FontDescriptor")
        .map(|descriptor| {
            let name = descriptor
                .get(b"FontName")
                .and_then(lopdf::Object::as_name)
                .expect("FontName");
            let file = descriptor
                .get(b"FontFile2")
                .and_then(lopdf::Object::as_reference)
                .and_then(|id| document.get_object(id))
                .and_then(lopdf::Object::as_stream)
                .expect("FontFile2");
            (
                String::from_utf8_lossy(name).into_owned(),
                file.decompressed_content().expect("Flate"),
            )
        })
        .collect()
}

fn page_contents(document: &lopdf::Document) -> String {
    document
        .get_pages()
        .values()
        .map(|&id| String::from_utf8(document.get_page_content(id).expect("Inhalt")).unwrap())
        .collect()
}

fn fill_operator(hex: &str) -> String {
    let [r, g, b] = brand::rgb_unit(hex).expect("Farbe");
    format!("{r:.3} {g:.3} {b:.3} rg")
}

#[test]
fn pdf_bettet_die_markenschrift_ein() {
    let bytes = pdf(
        &beispiel_datei("de"),
        ExportMode::Sop,
        PdfPage::default(),
        &style(),
    );
    let raw = String::from_utf8_lossy(&bytes);
    assert!(!raw.contains("/Helvetica"));
    assert!(!raw.contains("WinAnsiEncoding"));

    let document = lopdf::Document::load_mem(&bytes).expect("PDF");
    let fonts: Vec<_> = dicts_of_type(&document, b"Font")
        .filter(|dict| {
            dict.get(b"Subtype")
                .and_then(lopdf::Object::as_name)
                .is_ok_and(|name| name == b"Type0")
        })
        .collect();
    assert_eq!(fonts.len(), 2);
    for font in fonts {
        let encoding = font.get(b"Encoding").and_then(lopdf::Object::as_name);
        assert_eq!(encoding.expect("Encoding"), b"Identity-H");
        assert!(font.get(b"ToUnicode").is_ok());
    }

    let embedded = eingebettete_schriften(&document);
    let mut names: Vec<_> = embedded
        .iter()
        .map(|(name, _)| name.split_once('+').expect("Subset-Präfix"))
        .collect();
    names.sort_by_key(|(_, style)| *style);
    assert_eq!(
        names.iter().map(|(_, style)| *style).collect::<Vec<_>>(),
        [
            "AtkinsonHyperlegibleNext-Bold",
            "AtkinsonHyperlegibleNext-Regular"
        ]
    );
    for (tag, _) in &names {
        assert_eq!(tag.len(), 6);
        assert!(tag.bytes().all(|byte| byte.is_ascii_uppercase()));
    }
    for (_, data) in &embedded {
        let face = ttf_parser::Face::parse(data, 0).expect("TrueType");
        assert_eq!(
            face.names()
                .into_iter()
                .filter(|name| name.name_id == ttf_parser::name_id::FAMILY)
                .find_map(|name| name.to_string()),
            Some(brand::font().family.clone())
        );
    }
}

#[test]
fn pdf_zeigt_sonderzeichen_ausserhalb_von_winansi() {
    let document = lopdf::Document::load_mem(&pdf(
        &sonderzeichen_datei(),
        ExportMode::Sop,
        PdfPage::default(),
        &style(),
    ))
    .expect("PDF");
    // Der Markenschrift fehlen Pfeile; sie werden lesbar ersetzt statt verschluckt.
    let expected: Vec<String> = SONDERZEICHEN
        .iter()
        .map(|text| text.replace('→', "->"))
        .collect();
    let text = pdf_text(&document);
    assert!(text.contains("Übersicht -> Łódź"), "{text}");
    for line in &expected {
        assert!(text.contains(line.as_str()), "{line} fehlt in {text}");
    }
    assert_eq!(
        text.matches('?').count(),
        expected.concat().matches('?').count(),
        "Ersatzzeichen in {text}"
    );

    let regular = eingebettete_schriften(&document)
        .into_iter()
        .find(|(name, _)| name.ends_with("-Regular"))
        .expect("Regular");
    let face = ttf_parser::Face::parse(&regular.1, 0).expect("TrueType");
    for character in expected.concat().chars().filter(|c| !c.is_whitespace()) {
        let glyph = face.glyph_index(character).expect("Glyphe");
        assert!(
            face.glyph_bounding_box(glyph).is_some(),
            "Umriss für {character} fehlt"
        );
    }
}

#[test]
fn pdf_nutzt_markenfarben() {
    let custom = ExportStyle {
        brand: "ember".to_owned(),
        accent: "#10B981".to_owned(),
        credit: true,
    };
    let document = lopdf::Document::load_mem(&pdf(
        &beispiel_datei("de"),
        ExportMode::Sop,
        PdfPage::default(),
        &custom,
    ))
    .expect("PDF");
    let contents = page_contents(&document);
    let palette = brand::brand_palette("ember", Skin::Light, "#10B981");
    for (token, hex) in [
        ("ink", &palette.ink),
        ("accent", &palette.accent),
        ("on_accent", &palette.on_accent),
    ] {
        assert!(contents.contains(&fill_operator(hex)), "{token} fehlt");
    }
    let sage = brand::brand_palette(brand::default_brand(), Skin::Light, "");
    assert!(!contents.contains(&fill_operator(&sage.accent)));
}

#[test]
fn pdf_credit_folgt_der_einstellung() {
    let mut file = beispiel_datei("es");
    let template = file.guide.steps[0].clone();
    file.guide.steps = (0..12)
        .map(|index| {
            let mut step = template.clone();
            step.id = format!("step-{index:04}");
            file.images.insert(step.id.clone(), solid_rgba_png(160, 90));
            step
        })
        .collect();

    let with =
        lopdf::Document::load_mem(&pdf(&file, ExportMode::Sop, PdfPage::default(), &style()))
            .expect("PDF");
    let pages = with.get_pages();
    assert!(pages.len() > 1);
    for &page in pages.keys() {
        let text = with.extract_text(&[page]).expect("Text");
        assert!(text.contains(credit("es")), "Seite {page}: {text}");
    }

    let without = lopdf::Document::load_mem(&pdf(
        &file,
        ExportMode::Sop,
        PdfPage::default(),
        &ohne_credit(),
    ))
    .expect("PDF");
    let text = pdf_text(&without);
    assert!(!text.contains(credit("es")), "{text}");
    assert!(!text.contains("Hows"), "{text}");
}

#[test]
fn pdf_ist_gueltig_mit_text_und_eingebranntem_bild() {
    let mut file = beispiel_datei("de");
    file.images
        .insert("step-0001".to_owned(), solid_rgba_png(64, 36));
    file.guide.steps[0].overlays = vec![Overlay::Rect {
        id: "o1".to_owned(),
        color: marks().color.clone(),
        stroke: marks().stroke,
        x: 0.2,
        y: 0.2,
        w: 0.5,
        h: 0.4,
    }];

    let bytes = pdf(&file, ExportMode::Sop, PdfPage::default(), &style());
    assert!(bytes.starts_with(b"%PDF-1.4"));
    let document = lopdf::Document::load_mem(&bytes).expect("gültiges PDF");
    assert_eq!(document.get_pages().len(), 1);
    assert_eq!(image_xobject_count(&document), 1);

    let text = pdf_text(&document);
    assert!(text.contains("Netzlaufwerk verbinden"), "{text}");
    assert!(text.contains("Zur Freigabe"), "{text}");
    assert!(text.contains("Projekte"), "{text}");
    // SOP ohne technische Details:
    assert!(!text.contains("10.0.26100"));
    // Keine externen Referenzen:
    let raw = String::from_utf8_lossy(&bytes);
    assert!(!raw.contains("http://") && !raw.contains("https://"));
}

#[test]
fn pdf_bug_report_enthaelt_metadaten_und_ueberspringt_kaputte_bilder() {
    // `PNG` im Beispiel ist kein dekodierbares Bild: Schritt bleibt, Bild entfällt.
    let file = beispiel_datei("de");
    let document = lopdf::Document::load_mem(&pdf(
        &file,
        ExportMode::BugReport,
        PdfPage::default(),
        &style(),
    ))
    .expect("PDF");
    assert_eq!(image_xobject_count(&document), 0);
    let text = pdf_text(&document);
    assert!(text.contains("10.0.26100"), "{text}");
    assert!(text.contains("AutomationId=btnSave"), "{text}");
}

#[test]
fn pdf_bricht_lange_anleitungen_auf_mehrere_seiten_um() {
    let mut file = beispiel_datei("en");
    let template = file.guide.steps[0].clone();
    file.guide.steps = (0..12)
        .map(|index| {
            let mut step = template.clone();
            step.id = format!("step-{index:04}");
            step.text = StepText::generated(format!("Step number {index} with a screenshot"));
            file.images.insert(step.id.clone(), solid_rgba_png(160, 90));
            step
        })
        .collect();

    let document =
        lopdf::Document::load_mem(&pdf(&file, ExportMode::Sop, PdfPage::default(), &style()))
            .expect("PDF");
    assert!(document.get_pages().len() > 1);
    assert_eq!(image_xobject_count(&document), 12);
    let text = pdf_text(&document);
    assert!(text.contains("Step number 11"), "{text}");
}

#[test]
fn write_pdf_export_schreibt_datei() {
    let path =
        std::env::temp_dir().join(format!("steps-pdf-export-test-{}.pdf", std::process::id()));
    let _ = std::fs::remove_file(&path);
    write_pdf_export(
        &beispiel_datei("de"),
        ExportMode::Sop,
        PdfPage::default(),
        &style(),
        &path,
    )
    .expect("Export");
    let bytes = std::fs::read(&path).expect("PDF");
    assert!(bytes.ends_with(b"%%EOF\n"));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn pdf_nutzt_das_eingestellte_papierformat() {
    let page = PdfPage {
        paper: Paper::Letter,
        margin_mm: 25,
    };
    let document =
        lopdf::Document::load_mem(&pdf(&beispiel_datei("en"), ExportMode::Sop, page, &style()))
            .expect("PDF");
    let (_, page_id) = document.get_pages().into_iter().next().expect("eine Seite");
    let media_box = document
        .get_object(page_id)
        .and_then(lopdf::Object::as_dict)
        .and_then(|page| page.get(b"MediaBox"))
        .and_then(lopdf::Object::as_array)
        .expect("MediaBox");
    let size: Vec<f32> = media_box
        .iter()
        .map(|value| value.as_float().expect("Zahl"))
        .collect();
    assert_eq!(size, vec![0.0, 0.0, 612.0, 792.0]);
}

#[test]
fn vorschaubild_ist_verkleinert_und_behaelt_das_seitenverhaeltnis() {
    let uri = thumbnail_data_uri(&solid_rgba_png(1600, 900), 320).expect("Vorschau");
    let base64 = uri.strip_prefix("data:image/png;base64,").expect("PNG-URI");
    let decoded = decode_base64(base64);
    let thumbnail = image::load_from_memory(&decoded).expect("lesbar");
    assert_eq!((thumbnail.width(), thumbnail.height()), (320, 180));

    let small = thumbnail_data_uri(&solid_rgba_png(200, 100), 320).expect("Vorschau");
    let decoded = decode_base64(
        small
            .strip_prefix("data:image/png;base64,")
            .expect("PNG-URI"),
    );
    assert_eq!(
        image::load_from_memory(&decoded).expect("lesbar").width(),
        200
    );

    assert_eq!(thumbnail_data_uri(PNG, 320), None);
}

fn decode_base64(text: &str) -> Vec<u8> {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut bits = 0u32;
    let mut count = 0;
    let mut out = Vec::new();
    for byte in text.bytes().filter(|&b| b != b'=') {
        let value = ALPHABET.iter().position(|&c| c == byte).expect("base64");
        bits = (bits << 6) | u32::try_from(value).expect("6 Bit");
        count += 6;
        if count >= 8 {
            count -= 8;
            out.push(u8::try_from(bits >> count).expect("8 Bit"));
            bits &= (1 << count) - 1;
        }
    }
    out
}
