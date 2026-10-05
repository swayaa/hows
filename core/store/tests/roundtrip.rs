//! Roundtrip- und Fehlerfall-Tests für das `.steps`-Dateiformat.

use std::io::{Cursor, Write};

use steps_store::{
    marks, read_guide_from, Action, ElementInfo, Environment, Guide, Monitor, Overlay, Position,
    Preview, ScrollDirection, Step, StepText, StepsFile, StoreError, SCHEMA_VERSION,
};

/// Minimales, gültiges 1x1-PNG (nur als Platzhalter-Bytes; der Store
/// interpretiert Bildinhalte nicht).
const FAKE_PNG: &[u8] = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 1, 2, 3];

fn step(id: &str, action: Action, text: &str) -> Step {
    Step {
        id: id.to_owned(),
        action,
        timestamp_ms: 1_756_857_600_000,
        position: Some(Position { x: -120, y: 480 }),
        monitor: Some(Monitor {
            index: 1,
            x: -2560,
            y: 0,
            width: 2560,
            height: 1440,
            scale_factor: 1.5,
        }),
        element: Some(ElementInfo {
            name: Some("Speichern".to_owned()),
            control_type: Some("Button".to_owned()),
            automation_id: Some("btnSave".to_owned()),
            window_title: Some("Einstellungen".to_owned()),
            app_name: Some("Beispiel-App".to_owned()),
            ..ElementInfo::default()
        }),
        text: StepText::generated(text),
        overlays: sample_overlays(),
        crop: None,
    }
}

fn sample_overlays() -> Vec<Overlay> {
    let marks = marks();
    vec![
        Overlay::Rect {
            id: "o1".to_owned(),
            color: marks.color.clone(),
            stroke: marks.stroke,
            x: 0.12,
            y: 0.20,
            w: 0.30,
            h: 0.18,
        },
        Overlay::Arrow {
            id: "o2".to_owned(),
            color: marks.color.clone(),
            stroke: marks.stroke,
            x1: 0.40,
            y1: 0.50,
            x2: 0.62,
            y2: 0.35,
        },
        Overlay::Pen {
            id: "o3".to_owned(),
            color: marks.color.clone(),
            stroke: marks.pen_stroke(marks.stroke),
            points: vec![[0.10, 0.10], [0.12, 0.14], [0.15, 0.13]],
        },
        Overlay::Highlight {
            id: "o4".to_owned(),
            color: marks.highlight_color.clone(),
            opacity: marks.highlight_opacity,
            x: 0.20,
            y: 0.55,
            w: 0.35,
            h: 0.08,
        },
        Overlay::Text {
            id: "o5".to_owned(),
            color: marks.color.clone(),
            size: marks.text_size,
            x: 0.22,
            y: 0.58,
            text: "Hier klicken".to_owned(),
        },
        Overlay::Circle {
            id: "o6".to_owned(),
            color: marks.color.clone(),
            stroke: marks.stroke,
            cx: 0.40,
            cy: 0.35,
            r: 0.08,
        },
        Overlay::Blur {
            id: "o7".to_owned(),
            x: 0.10,
            y: 0.70,
            w: 0.25,
            h: 0.12,
        },
    ]
}

fn full_guide() -> Guide {
    let mut guide = Guide::new("Testanleitung", 1_756_857_600_000, "de");
    guide.description = Some("Beschreibung".to_owned());
    guide.environment = Some(Environment {
        os: Some("Windows".to_owned()),
        os_version: Some("10.0.26100".to_owned()),
    });
    guide.steps = vec![
        step("s1", Action::Click, "Klick auf „Speichern“"),
        step("s2", Action::DoubleClick, "Doppelklick"),
        step("s3", Action::RightClick, "Rechtsklick"),
        step(
            "s4",
            Action::KeyCombo {
                combo: "Ctrl+S".to_owned(),
            },
            "Tastenkombination Strg+S",
        ),
        step(
            "s5",
            Action::Scroll {
                direction: ScrollDirection::Down,
                amount: 7,
            },
            "Nach unten gescrollt",
        ),
        step("s6", Action::TextInput, "Text eingegeben"),
        step("s7", Action::ManualNote, "Manuelle Notiz"),
    ];
    guide
}

#[test]
fn roundtrip_erhaelt_guide_und_bilder() {
    let mut file = StepsFile::new(full_guide());
    file.images.insert("s1".to_owned(), FAKE_PNG.to_vec());
    file.images.insert("s2".to_owned(), vec![42; 1024]);

    let mut buffer = Cursor::new(Vec::new());
    file.write_to(&mut buffer).expect("Schreiben");

    buffer.set_position(0);
    let read_back = StepsFile::read_from(buffer).expect("Lesen");

    assert_eq!(read_back, file);
}

#[test]
fn roundtrip_leerer_guide() {
    let file = StepsFile::new(Guide::new("Leer", 0, "en"));

    let mut buffer = Cursor::new(Vec::new());
    file.write_to(&mut buffer).expect("Schreiben");

    buffer.set_position(0);
    let read_back = StepsFile::read_from(buffer).expect("Lesen");

    assert_eq!(read_back.guide.schema_version, SCHEMA_VERSION);
    assert_eq!(read_back, file);
}

#[test]
fn save_und_open_ueber_dateisystem() {
    let dir = std::env::temp_dir().join(format!("steps-store-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("Temp-Verzeichnis");
    let path = dir.join("anleitung.steps");
    StepsFile::new(Guide::new("Alt", 0, "en"))
        .save(&path)
        .expect("Speichern");

    let mut file = StepsFile::new(full_guide());
    file.images.insert("s1".to_owned(), FAKE_PNG.to_vec());
    file.save(&path).expect("Speichern");

    let read_back = StepsFile::open(&path).expect("Öffnen");
    assert_eq!(read_back, file);
    let names: Vec<_> = std::fs::read_dir(&dir)
        .expect("Ordner lesen")
        .map(|entry| entry.expect("Eintrag").file_name())
        .collect();
    assert_eq!(names, ["anleitung.steps"]);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn override_hat_vorrang_vor_generiertem_text() {
    let mut text = StepText::generated("generiert");
    assert_eq!(text.effective(), "generiert");
    text.override_text = Some("überschrieben".to_owned());
    assert_eq!(text.effective(), "überschrieben");
}

#[test]
fn fehlendes_guide_json_wird_gemeldet() {
    // ZIP mit nur einem Bild, ohne guide.json.
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buffer);
        zip.start_file("images/s1.png", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(FAKE_PNG).unwrap();
        zip.finish().unwrap();
    }
    buffer.set_position(0);

    let err = StepsFile::read_from(buffer).unwrap_err();
    assert!(matches!(err, StoreError::MissingGuide), "war: {err:?}");
}

#[test]
fn neuere_schema_version_wird_abgelehnt() {
    let mut guide_json = serde_json::to_value(full_guide()).unwrap();
    guide_json["schema_version"] = serde_json::json!(999);

    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buffer);
        zip.start_file("guide.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(serde_json::to_string(&guide_json).unwrap().as_bytes())
            .unwrap();
        zip.finish().unwrap();
    }
    buffer.set_position(0);

    let err = StepsFile::read_from(buffer).unwrap_err();
    assert!(
        matches!(
            err,
            StoreError::UnsupportedSchemaVersion {
                found: 999,
                supported: SCHEMA_VERSION,
            }
        ),
        "war: {err:?}"
    );
}

#[test]
fn vorschau_liest_guide_und_nur_das_erste_bild() {
    let mut file = StepsFile::new(full_guide());
    // s1 und s2 haben kein Bild, also ist s3 das erste.
    file.images.insert("s3".to_owned(), FAKE_PNG.to_vec());
    file.images.insert("s5".to_owned(), vec![7; 64]);
    let mut buffer = Cursor::new(Vec::new());
    file.write_to(&mut buffer).expect("Schreiben");

    buffer.set_position(0);
    let preview = Preview::read_from(&mut buffer).expect("Vorschau");
    assert_eq!(preview.guide, file.guide);
    assert_eq!(preview.first_image.as_deref(), Some(FAKE_PNG));

    buffer.set_position(0);
    assert_eq!(read_guide_from(buffer).expect("Guide"), file.guide);
}

#[test]
fn vorschau_ohne_bilder_hat_kein_erstes_bild() {
    let file = StepsFile::new(full_guide());
    let mut buffer = Cursor::new(Vec::new());
    file.write_to(&mut buffer).expect("Schreiben");
    buffer.set_position(0);
    assert_eq!(
        Preview::read_from(buffer).expect("Vorschau").first_image,
        None
    );
}

#[test]
fn kein_zip_wird_als_fehler_gemeldet() {
    let buffer = Cursor::new(b"kein zip".to_vec());
    let err = StepsFile::read_from(buffer).unwrap_err();
    assert!(matches!(err, StoreError::Zip(_)), "war: {err:?}");
}

#[test]
fn unbekannte_json_felder_werden_toleriert() {
    let mut guide_json = serde_json::to_value(Guide::new("T", 1, "de")).unwrap();
    guide_json["zukunftsfeld"] = serde_json::json!({"neu": true});

    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buffer);
        zip.start_file("guide.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(serde_json::to_string(&guide_json).unwrap().as_bytes())
            .unwrap();
        zip.finish().unwrap();
    }
    buffer.set_position(0);

    let file = StepsFile::read_from(buffer).expect("Lesen");
    assert_eq!(file.guide.title, "T");
}

#[test]
fn action_serialisierung_nutzt_snake_case_tags() {
    let json = serde_json::to_value(Action::KeyCombo {
        combo: "Ctrl+S".to_owned(),
    })
    .unwrap();
    assert_eq!(json["type"], "key_combo");
    assert_eq!(json["combo"], "Ctrl+S");

    let json = serde_json::to_value(Action::DoubleClick).unwrap();
    assert_eq!(json["type"], "double_click");
}

#[test]
fn fehlende_overlays_gelten_als_leer() {
    let mut guide_json = serde_json::to_value(Guide::new("T", 1, "de")).unwrap();
    guide_json["steps"] = serde_json::json!([{
        "id": "s1",
        "action": { "type": "click" },
        "timestamp_ms": 1,
        "text": { "generated": "x" }
    }]);

    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buffer);
        zip.start_file("guide.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(serde_json::to_string(&guide_json).unwrap().as_bytes())
            .unwrap();
        zip.finish().unwrap();
    }
    buffer.set_position(0);

    let file = StepsFile::read_from(buffer).expect("Lesen");
    assert_eq!(file.guide.steps[0].overlays.len(), 0);
}

#[test]
fn overlays_serialisieren_normalisierte_koordinaten() {
    let overlay = Overlay::Rect {
        id: "o1".to_owned(),
        color: "#2563EB".to_owned(),
        stroke: 2.5,
        x: 0.12,
        y: 0.20,
        w: 0.30,
        h: 0.18,
    };
    let json = serde_json::to_value(&overlay).unwrap();
    assert_eq!(json["type"], "rect");
    assert_eq!(json["color"], "#2563EB");
    assert_eq!(json["stroke"], 2.5);
    assert_eq!(json["x"], 0.12);
}

#[test]
fn unbekannte_overlay_typen_werden_uebersprungen() {
    let mut guide_json = serde_json::to_value(Guide::new("T", 1, "de")).unwrap();
    guide_json["steps"] = serde_json::json!([{
        "id": "s1",
        "action": { "type": "click" },
        "timestamp_ms": 1,
        "text": { "generated": "x" },
        "overlays": [
            { "id": "o1", "type": "rect", "color": "#E11D48", "stroke": 2.5,
              "x": 0.1, "y": 0.1, "w": 0.2, "h": 0.2 },
            { "id": "future", "type": "sparkle", "color": "#000000" }
        ]
    }]);

    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buffer);
        zip.start_file("guide.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(serde_json::to_string(&guide_json).unwrap().as_bytes())
            .unwrap();
        zip.finish().unwrap();
    }
    buffer.set_position(0);

    let file = StepsFile::read_from(buffer).expect("Lesen");
    assert_eq!(file.guide.steps[0].overlays.len(), 1);
    assert!(matches!(
        file.guide.steps[0].overlays[0],
        Overlay::Rect { .. }
    ));
}
