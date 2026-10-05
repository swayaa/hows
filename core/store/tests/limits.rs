//! Grenzen beim Lesen fremder `.steps`-Dateien: ungültige Schritt-IDs,
//! übergroße Container und extreme Markierungen machen die ganze Datei
//! unlesbar, normale Dateien laden weiter.

use std::io::{Cursor, Write};

use serde_json::json;
use steps_store::limits::{
    MAX_GUIDE_JSON_BYTES, MAX_IMAGE_BYTES, MAX_OVERLAYS_PER_STEP, MAX_PEN_POINTS, MAX_STEPS,
    MAX_TEXT_CHARS, MAX_ZIP_ENTRIES,
};
use steps_store::{
    marks, read_guide_from, Action, Guide, Monitor, Overlay, Preview, ReadFailure, Step, StepText,
    StepsFile, StoreError,
};
use zip::write::SimpleFileOptions;
use zip::CompressionMethod;

/// PNG-Signatur und `IHDR` mit der angegebenen Fläche, danach Platzhalter-Bytes.
fn png_header(width: u32, height: u32) -> Vec<u8> {
    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A, 0, 0, 0, 13];
    png.extend_from_slice(b"IHDR");
    png.extend_from_slice(&width.to_be_bytes());
    png.extend_from_slice(&height.to_be_bytes());
    png.extend_from_slice(&[8, 6, 0, 0, 0, 1, 2, 3, 4]);
    png
}

fn step(id: &str) -> Step {
    Step {
        id: id.to_owned(),
        action: Action::Click,
        timestamp_ms: 1,
        position: None,
        monitor: None,
        element: None,
        text: StepText::generated("Klick"),
        overlays: Vec::new(),
        crop: None,
    }
}

fn zip_with(entries: &[(&str, &[u8])]) -> Cursor<Vec<u8>> {
    let mut buffer = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(&mut buffer);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    for (name, bytes) in entries {
        zip.start_file(*name, options).unwrap();
        zip.write_all(bytes).unwrap();
    }
    zip.finish().unwrap();
    buffer.set_position(0);
    buffer
}

fn guide_json(steps: &serde_json::Value) -> Vec<u8> {
    let mut guide = serde_json::to_value(Guide::new("T", 1, "de")).unwrap();
    guide["steps"] = steps.clone();
    serde_json::to_vec(&guide).unwrap()
}

fn step_json(id: &str, overlays: &serde_json::Value) -> serde_json::Value {
    json!({
        "id": id,
        "action": { "type": "click" },
        "timestamp_ms": 1,
        "text": { "generated": "x" },
        "overlays": overlays,
    })
}

/// Jeder Leseweg lehnt den Container ab und meldet ihn als beschädigt.
fn assert_rejected(container: &Cursor<Vec<u8>>, case: &str) {
    let failures = [
        StepsFile::read_from(container.clone()).err(),
        Preview::read_from(container.clone()).err(),
        read_guide_from(container.clone()).err(),
    ];
    for failure in failures {
        let error: StoreError = failure.unwrap_or_else(|| panic!("{case}: wurde geladen"));
        assert_eq!(
            error.read_failure(),
            ReadFailure::Corrupt,
            "{case}: {error}"
        );
    }
}

#[test]
fn normale_datei_mit_step_0001_und_step_10000_bleibt_lesbar() {
    let marks = marks();
    let mut guide = Guide::new("Normal", 1, "de");
    let mut first = step("step-0001");
    first.overlays = vec![
        Overlay::Rect {
            id: "o1".to_owned(),
            color: marks.color.clone(),
            stroke: marks.stroke,
            x: -0.4,
            y: 0.2,
            w: 0.6,
            h: 0.3,
        },
        Overlay::Pen {
            id: "o2".to_owned(),
            color: marks.color.clone(),
            stroke: marks.pen_stroke(marks.stroke),
            points: vec![[0.1, 0.1], [0.9, 1.4]],
        },
        Overlay::Text {
            id: "o3".to_owned(),
            color: marks.color.clone(),
            size: marks.text_resize.max,
            x: 0.5,
            y: 0.5,
            text: "Hier".to_owned(),
        },
        // Größter Radius, den der Ziehpunkt auf einem 16:9-Bild schreibt: Mitte bis Ecke.
        Overlay::Circle {
            id: "o4".to_owned(),
            color: marks.color.clone(),
            stroke: marks.stroke,
            cx: 0.5,
            cy: 0.5,
            r: 1.147,
        },
    ];
    guide.steps = vec![first, step("step-10000")];
    let mut file = StepsFile::new(guide);
    file.images
        .insert("step-0001".to_owned(), png_header(1920, 1080));
    file.images
        .insert("step-10000".to_owned(), png_header(7680, 4320));

    let mut buffer = Cursor::new(Vec::new());
    file.write_to(&mut buffer).expect("Schreiben");

    buffer.set_position(0);
    assert_eq!(StepsFile::read_from(buffer.clone()).expect("Lesen"), file);
    let preview = Preview::read_from(buffer.clone()).expect("Vorschau");
    assert_eq!(preview.first_image, Some(png_header(1920, 1080)));
    assert_eq!(read_guide_from(buffer).expect("Guide"), file.guide);
}

#[test]
fn ids_mit_pfadzeichen_werden_abgelehnt() {
    let too_long = "s".repeat(65);
    for id in [
        "a/b",
        r"a\b",
        r"..\..\victim",
        "../../victim",
        "..",
        ".",
        "",
        r"C:\victim",
        "step 1",
        "CON",
        "nul",
        "Com1",
        too_long.as_str(),
    ] {
        let container = zip_with(&[(
            "guide.json",
            &guide_json(&json!([step_json(id, &json!([]))])),
        )]);
        assert_rejected(&container, id);
    }
}

#[test]
fn doppelte_ids_werden_abgelehnt() {
    let steps = json!([
        step_json("step-0001", &json!([])),
        step_json("step-0001", &json!([]))
    ]);
    assert_rejected(&zip_with(&[("guide.json", &guide_json(&steps))]), "doppelt");
}

#[test]
fn ids_die_sich_nur_in_der_schreibweise_unterscheiden_werden_abgelehnt() {
    let steps = json!([
        step_json("step-A", &json!([])),
        step_json("step-a", &json!([]))
    ]);
    assert_rejected(
        &zip_with(&[("guide.json", &guide_json(&steps))]),
        "Großschreibung",
    );
    let unique = json!([
        step_json("step-A", &json!([])),
        step_json("step-B", &json!([]))
    ]);
    read_guide_from(zip_with(&[("guide.json", &guide_json(&unique))])).expect("eindeutig");
    assert_rejected(
        &zip_with(&[(
            "guide.json",
            &guide_json(&json!([step_json("../victim", &json!([]))])),
        )]),
        "Pfad",
    );
}

#[test]
fn zu_grosses_guide_json_wird_abgelehnt() {
    let mut guide = serde_json::to_value(Guide::new("T", 1, "de")).unwrap();
    let padding = usize::try_from(MAX_GUIDE_JSON_BYTES).unwrap();
    guide["padding"] = json!(" ".repeat(padding));
    let container = zip_with(&[("guide.json", &serde_json::to_vec(&guide).unwrap())]);
    assert_rejected(&container, "guide.json über der Grenze");
}

#[test]
fn zu_viele_zip_eintraege_werden_abgelehnt() {
    let guide = guide_json(&json!([]));
    let names: Vec<String> = (0..MAX_ZIP_ENTRIES)
        .map(|index| format!("extra/{index}"))
        .collect();
    let mut entries: Vec<(&str, &[u8])> = vec![("guide.json", &guide)];
    entries.extend(names.iter().map(|name| (name.as_str(), &b""[..])));
    assert_rejected(&zip_with(&entries), "zu viele Einträge");
}

#[test]
fn zu_grosses_bild_wird_abgelehnt() {
    let guide = guide_json(&json!([step_json("step-0001", &json!([]))]));
    let mut huge = png_header(1920, 1080);
    huge.resize(usize::try_from(MAX_IMAGE_BYTES).unwrap() + 1, 0);
    let container = zip_with(&[("guide.json", &guide), ("images/step-0001.png", &huge)]);
    for result in [
        StepsFile::read_from(container.clone()).err(),
        Preview::read_from(container).err(),
    ] {
        let error = result.expect("Bild über der Byte-Grenze wurde geladen");
        assert_eq!(error.read_failure(), ReadFailure::Corrupt, "{error}");
    }
}

#[test]
fn bild_mit_zu_grosser_flaeche_wird_abgelehnt() {
    let guide = guide_json(&json!([step_json("step-0001", &json!([]))]));
    for (width, height) in [(20_000, 100), (100, 20_000), (10_000, 10_000)] {
        let container = zip_with(&[
            ("guide.json", &guide),
            ("images/step-0001.png", &png_header(width, height)),
        ]);
        assert!(
            StepsFile::read_from(container.clone()).is_err(),
            "{width}x{height}"
        );
        assert!(Preview::read_from(container).is_err(), "{width}x{height}");
    }
}

#[test]
fn extreme_markierungen_werden_abgelehnt() {
    let rect = |x: f64, w: f64, stroke: f64| {
        json!({ "type": "rect", "id": "o1", "color": "#E11D48", "stroke": stroke,
                "x": x, "y": 0.1, "w": w, "h": 0.1 })
    };
    let pen_points: Vec<[f64; 2]> = vec![[0.5, 0.5]; MAX_PEN_POINTS + 1];
    let cases = [
        ("x 1e300", json!([rect(1e300, 0.1, 2.5)])),
        ("x -1e300", json!([rect(-1e300, 0.1, 2.5)])),
        ("w 1e9", json!([rect(0.1, 1e9, 2.5)])),
        ("stroke 1e6", json!([rect(0.1, 0.1, 1e6)])),
        ("stroke negativ", json!([rect(0.1, 0.1, -1.0)])),
        (
            "Pfeil 1e300",
            json!([{ "type": "arrow", "id": "a", "color": "#E11D48", "stroke": 2.5,
                     "x1": 0.1, "y1": 0.1, "x2": 1e300, "y2": 0.2 }]),
        ),
        (
            "Kreis r 1e9",
            json!([{ "type": "circle", "id": "c", "color": "#E11D48", "stroke": 2.5,
                     "cx": 0.5, "cy": 0.5, "r": 1e9 }]),
        ),
        (
            "Stiftpunkt 1e300",
            json!([{ "type": "pen", "id": "p", "color": "#E11D48", "stroke": 3.0,
                     "points": [[0.1, 0.1], [1e300, 0.2]] }]),
        ),
        (
            "zu viele Stiftpunkte",
            json!([{ "type": "pen", "id": "p", "color": "#E11D48", "stroke": 3.0,
                     "points": pen_points }]),
        ),
        (
            "Textgröße 1e9",
            json!([{ "type": "text", "id": "t", "color": "#E11D48", "size": 1e9,
                     "x": 0.1, "y": 0.1, "text": "Hier" }]),
        ),
        (
            "zu langer Text",
            json!([{ "type": "text", "id": "t", "color": "#E11D48", "size": 14.0,
                     "x": 0.1, "y": 0.1, "text": "x".repeat(MAX_TEXT_CHARS + 1) }]),
        ),
        (
            "Deckkraft 5",
            json!([{ "type": "highlight", "id": "h", "color": "#FDE047", "opacity": 5.0,
                     "x": 0.1, "y": 0.1, "w": 0.2, "h": 0.2 }]),
        ),
        (
            "Unschärfe 1e300",
            json!([{ "type": "blur", "id": "b", "x": 0.1, "y": 0.1, "w": 1e300, "h": 0.2 }]),
        ),
        (
            "zu viele Markierungen",
            serde_json::Value::Array(vec![rect(0.1, 0.1, 2.5); MAX_OVERLAYS_PER_STEP + 1]),
        ),
    ];
    for (case, overlays) in cases {
        let container = zip_with(&[(
            "guide.json",
            &guide_json(&json!([step_json("step-0001", &overlays)])),
        )]);
        assert_rejected(&container, case);
    }
}

/// Ein Bild mit gültigem Kopf und `len` Bytes; der Rest bleibt unberührt.
fn png_of_len(len: u64) -> Vec<u8> {
    let header = png_header(1920, 1080);
    let mut png = vec![0; usize::try_from(len).unwrap()];
    png[..header.len()].copy_from_slice(&header);
    png
}

fn file_with_step(edit: impl FnOnce(&mut Step)) -> StepsFile {
    let mut first = step("step-0001");
    edit(&mut first);
    let mut guide = Guide::new("T", 1, "de");
    guide.steps = vec![first];
    StepsFile::new(guide)
}

#[test]
fn was_hows_schreibt_verletzt_keine_lesegrenze() {
    let rect = |x: f64| Overlay::Rect {
        id: "o1".to_owned(),
        color: "#E11D48".to_owned(),
        stroke: 2.5,
        x,
        y: 0.1,
        w: 0.1,
        h: 0.1,
    };
    let mut cases: Vec<(&str, StepsFile)> = vec![
        (
            "Pfad als Schritt-ID",
            file_with_step(|s| s.id = r"..\x".to_owned()),
        ),
        (
            "Gerätename als Schritt-ID",
            file_with_step(|s| s.id = "CON".to_owned()),
        ),
        (
            "Markierung außerhalb",
            file_with_step(|s| s.overlays = vec![rect(1e300)]),
        ),
        (
            "Markierung NaN",
            file_with_step(|s| s.overlays = vec![rect(f64::NAN)]),
        ),
        (
            "zu viele Markierungen",
            file_with_step(|s| s.overlays = vec![rect(0.1); MAX_OVERLAYS_PER_STEP + 1]),
        ),
        (
            "zu langer Text",
            file_with_step(|s| s.text = StepText::generated("x".repeat(MAX_TEXT_CHARS + 1))),
        ),
        (
            "Skalierung NaN",
            file_with_step(|s| {
                s.monitor = Some(Monitor {
                    index: 0,
                    x: 0,
                    y: 0,
                    width: 1920,
                    height: 1080,
                    scale_factor: f64::NAN,
                });
            }),
        ),
    ];
    cases.extend(unwritable_guides());
    cases.extend(unwritable_images());

    for (case, file) in cases {
        let mut buffer = Cursor::new(Vec::new());
        let error = file
            .write_to(&mut buffer)
            .expect_err(&format!("{case}: wurde geschrieben"));
        assert!(buffer.get_ref().is_empty(), "{case}: Bytes geschrieben");
        assert!(
            !matches!(error, StoreError::Io(_) | StoreError::Zip(_)),
            "{case}: {error}"
        );
    }
}

fn unwritable_guides() -> Vec<(&'static str, StepsFile)> {
    let mut cases = Vec::new();
    let mut doubled = file_with_step(|_| {});
    doubled.guide.steps.push(step("step-0001"));
    cases.push(("doppelte Schritt-ID", doubled));
    let mut mixed_case = file_with_step(|step| "step-A".clone_into(&mut step.id));
    mixed_case.guide.steps.push(step("step-a"));
    cases.push(("Schritt-ID nur in der Schreibweise verschieden", mixed_case));
    let mut too_many = file_with_step(|_| {});
    too_many.guide.steps = (0..=MAX_STEPS).map(|i| step(&format!("s{i}"))).collect();
    cases.push(("zu viele Schritte", too_many));
    let mut newer = file_with_step(|_| {});
    newer.guide.schema_version += 1;
    cases.push(("neuere Schema-Version", newer));
    let mut padded = file_with_step(|_| {});
    let long_text = "x".repeat(MAX_TEXT_CHARS);
    padded.guide.steps = (0..MAX_STEPS)
        .map(|i| {
            let mut s = step(&format!("s{i}"));
            s.text = StepText::generated(long_text.clone());
            s
        })
        .collect();
    cases.push(("guide.json über der Grenze", padded));
    cases
}

fn unwritable_images() -> Vec<(&'static str, StepsFile)> {
    let mut cases = Vec::new();
    let image_cases: [(&str, &str, Vec<u8>); 4] = [
        ("Pfad als Bildname", "../victim", png_header(10, 10)),
        ("leerer Bildname", "", png_header(10, 10)),
        (
            "Bild mit zu großer Fläche",
            "step-0001",
            png_header(20_000, 20_000),
        ),
        (
            "Bild über der Byte-Grenze",
            "step-0001",
            png_of_len(MAX_IMAGE_BYTES + 1),
        ),
    ];
    for (case, key, png) in image_cases {
        let mut file = file_with_step(|_| {});
        file.images.insert(key.to_owned(), png);
        cases.push((case, file));
    }
    cases
}

#[test]
fn abgelehntes_speichern_laesst_vorhandene_datei_stehen() {
    let dir = std::env::temp_dir().join(format!("steps-store-limits-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("Temp-Verzeichnis");
    let path = dir.join("anleitung.steps");
    let good = file_with_step(|_| {});
    good.save(&path).expect("Speichern");
    let before = std::fs::read(&path).expect("Lesen");

    let bad = file_with_step(|s| s.id = "a/b".to_owned());
    assert!(bad.save(&path).is_err());

    assert_eq!(std::fs::read(&path).expect("Lesen"), before);
    assert_eq!(StepsFile::open(&path).expect("Öffnen"), good);
    let names: Vec<_> = std::fs::read_dir(&dir)
        .expect("Ordner lesen")
        .map(|entry| entry.expect("Eintrag").file_name())
        .collect();
    assert_eq!(names, ["anleitung.steps"]);
    std::fs::remove_dir_all(&dir).ok();
}
