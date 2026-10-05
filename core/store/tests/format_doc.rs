//! Die JSON-Beispiele der Format-Doku (`docs/en/file-format.md` und
//! `docs/de/file-format.md`) müssen mit dem echten Reader ladbar sein, sonst
//! driftet die öffentliche Format-Doku vom Code ab.

use std::io::{Cursor, Write};

use steps_store::{Action, Overlay, StepsFile};

const FORMAT_DOCS: [(&str, &str); 2] = [
    ("en", include_str!("../../../docs/en/file-format.md")),
    ("de", include_str!("../../../docs/de/file-format.md")),
];

fn json_blocks(markdown: &str) -> Vec<&str> {
    markdown
        .split("```json\n")
        .skip(1)
        .filter_map(|rest| rest.split("\n```").next())
        .collect()
}

fn read_guide_json(json: &str) -> StepsFile {
    let mut buffer = Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buffer);
        zip.start_file("guide.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(json.as_bytes()).unwrap();
        zip.finish().unwrap();
    }
    buffer.set_position(0);
    StepsFile::read_from(buffer).expect("Beispiel der Format-Doku muss ladbar sein")
}

#[test]
fn format_doc_beispiele_sind_gueltige_guides() {
    for (language, doc) in FORMAT_DOCS {
        let blocks = json_blocks(doc);
        assert_eq!(
            blocks.len(),
            2,
            "{language}: Vollbeispiel und Minimalbeispiel erwartet"
        );

        let full = read_guide_json(blocks[0]);
        let steps = &full.guide.steps;
        assert_eq!(steps.len(), 2, "{language}");
        assert_eq!(steps[0].action, Action::Click, "{language}");
        assert!(
            matches!(steps[0].overlays[..], [Overlay::Rect { .. }]),
            "{language}"
        );
        assert_eq!(
            steps[1].action,
            Action::KeyCombo {
                combo: "Ctrl+S".to_owned()
            },
            "{language}"
        );
        assert_eq!(
            steps[1].text.effective(),
            "Save the file with Ctrl+S",
            "{language}"
        );

        let minimal = read_guide_json(blocks[1]);
        assert_eq!(minimal.guide.steps.len(), 1, "{language}");
        assert_eq!(
            minimal.guide.steps[0].action,
            Action::ManualNote,
            "{language}"
        );
    }
}
