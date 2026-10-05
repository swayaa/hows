//! Lesen und Schreiben des `.steps`-Dateiformats.
//!
//! Eine Aufnahme ist ein ZIP-Container (wie `.docx`) mit
//! `guide.json` (Schema, Metadaten, Schritte) und `images/{step-id}.png`
//! (Original-Screenshots). Siehe Spec §4.

mod files;
pub mod limits;
mod marks;
mod model;

pub use files::{unused_dir_path, unused_file_path, write_atomically};
pub use marks::{marks, ArrowHead, BlurBlock, Marks, TextBox, TextHalo, TextResize};
pub use model::{
    Action, Crop, ElementInfo, Environment, Guide, Monitor, Overlay, Position, ScrollDirection,
    Step, StepText, SCHEMA_VERSION,
};

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

/// Dateiname des Guide-Dokuments im Container.
pub const GUIDE_FILE_NAME: &str = "guide.json";
/// Verzeichnis der Original-Screenshots im Container.
pub const IMAGES_DIR: &str = "images";

/// Fehler beim Lesen oder Schreiben einer `.steps`-Datei. Der Text ist ein
/// englisches technisches Detail für Logs; Nutzer sehen die Übersetzung von
/// [`StoreError::read_failure`].
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// E/A-Fehler.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
    /// Ungültiger oder beschädigter ZIP-Container.
    #[error("invalid zip container: {0}")]
    Zip(#[from] zip::result::ZipError),
    /// `guide.json` ist kein gültiges JSON bzw. passt nicht zum Schema.
    #[error("invalid guide.json: {0}")]
    Json(#[from] serde_json::Error),
    /// Der Container enthält kein `guide.json`.
    #[error("container has no {GUIDE_FILE_NAME}")]
    MissingGuide,
    /// Die Datei verwendet eine neuere Schema-Version als unterstützt.
    #[error("schema version {found} is not supported (max {supported})")]
    UnsupportedSchemaVersion {
        /// In der Datei gefundene Version.
        found: u32,
        /// Höchste unterstützte Version.
        supported: u32,
    },
    /// Der Inhalt verletzt eine Regel oder Grenze aus [`limits`].
    #[error("invalid guide: {0}")]
    Invalid(String),
}

/// Warum sich eine `.steps`-Datei nicht öffnen ließ. [`ReadFailure::code`]
/// ist derselbe Code wie `ErrorCode` der App und `error.<code>` in `steps-i18n`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReadFailure {
    /// Die Datei gibt es nicht.
    NotFound,
    /// Die Datei ist beschädigt oder kein `.steps`-Container.
    Corrupt,
    /// Die Datei stammt aus einer neueren Version.
    NewerVersion,
}

impl ReadFailure {
    /// Alle Varianten.
    pub const ALL: [ReadFailure; 3] = [Self::NotFound, Self::Corrupt, Self::NewerVersion];

    /// Stabiler Code in `snake_case`.
    pub fn code(self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::Corrupt => "corrupt",
            Self::NewerVersion => "newer_version",
        }
    }
}

impl StoreError {
    /// Einordnung eines Fehlers beim Öffnen für die Meldung an Nutzer.
    pub fn read_failure(&self) -> ReadFailure {
        match self {
            Self::Io(error) if error.kind() == std::io::ErrorKind::NotFound => {
                ReadFailure::NotFound
            }
            Self::UnsupportedSchemaVersion { .. } => ReadFailure::NewerVersion,
            Self::Io(_) | Self::Zip(_) | Self::Json(_) | Self::MissingGuide | Self::Invalid(_) => {
                ReadFailure::Corrupt
            }
        }
    }
}

/// Ergebnis-Alias für Store-Operationen.
pub type Result<T> = std::result::Result<T, StoreError>;

/// Tage seit 1970-01-01 als (Jahr, Monat, Tag), proleptischer Gregorianischer
/// Kalender (Howard-Hinnant-Algorithmus, keine Datums-Dependency).
pub fn civil_from_days(days_since_epoch: i64) -> (i64, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    (year, month as u32, day as u32)
}

/// Eine vollständige `.steps`-Datei im Speicher: Guide plus Screenshots.
#[derive(Debug, Clone, PartialEq)]
pub struct StepsFile {
    /// Inhalt von `guide.json`.
    pub guide: Guide,
    /// PNG-Bytes je Schritt-ID (`images/{id}.png` im Container).
    pub images: BTreeMap<String, Vec<u8>>,
}

impl StepsFile {
    /// Erzeugt eine `.steps`-Struktur ohne Bilder.
    pub fn new(guide: Guide) -> Self {
        Self {
            guide,
            images: BTreeMap::new(),
        }
    }

    /// Pfad eines Screenshots im Container für eine Schritt-ID.
    pub fn image_path(step_id: &str) -> String {
        format!("{IMAGES_DIR}/{step_id}.png")
    }

    /// Schreibt den Container in einen beliebigen Writer.
    ///
    /// `guide.json` wird deflate-komprimiert; PNGs werden unkomprimiert
    /// abgelegt (bereits komprimiert, spart CPU). Vorher läuft dieselbe
    /// Prüfung wie beim Lesen: Was hier geschrieben wird, öffnet dieselbe
    /// Version wieder.
    pub fn write_to<W: Write + Seek>(&self, writer: W) -> Result<()> {
        let json = self.checked_guide_json()?;
        self.write_container(&json, writer)
    }

    /// Speichert den Container als Datei. Schlägt das Speichern fehl, bleibt
    /// eine vorhandene Datei unter `path` unverändert.
    pub fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let json = self.checked_guide_json()?;
        files::write_atomically(path.as_ref(), |writer| self.write_container(&json, writer))
    }

    /// Serialisiert `guide.json` und prüft den ganzen Inhalt gegen die
    /// Regeln des Lesewegs, bevor ein Byte geschrieben wird. Der Guide im
    /// Speicher wird selbst geprüft, weil der Leseweg ungültige Markierungen
    /// still verwirft.
    fn checked_guide_json(&self) -> Result<Vec<u8>> {
        limits::check_guide(&self.guide).map_err(StoreError::Invalid)?;
        if self.images.len() + 1 > limits::MAX_ZIP_ENTRIES {
            return Err(StoreError::Invalid(format!(
                "more than {} zip entries",
                limits::MAX_ZIP_ENTRIES
            )));
        }
        let json = serde_json::to_vec_pretty(&self.guide)?;
        if json.len() as u64 > limits::MAX_GUIDE_JSON_BYTES {
            return Err(StoreError::Invalid(format!(
                "{GUIDE_FILE_NAME} exceeds {} bytes",
                limits::MAX_GUIDE_JSON_BYTES
            )));
        }
        parse_guide(&json)?;
        let mut total = 0;
        for (step_id, png) in &self.images {
            if !limits::is_valid_step_id(step_id) {
                return Err(StoreError::Invalid(format!(
                    "image key {step_id:?} is not an allowed step id"
                )));
            }
            limits::check_image(png).map_err(StoreError::Invalid)?;
            limits::add_image_bytes(&mut total, png.len()).map_err(StoreError::Invalid)?;
        }
        Ok(json)
    }

    fn write_container<W: Write + Seek>(&self, json: &[u8], writer: W) -> Result<()> {
        let mut zip = ZipWriter::new(writer);

        let json_options =
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        zip.start_file(GUIDE_FILE_NAME, json_options)?;
        zip.write_all(json)?;

        let image_options =
            SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        for (step_id, png) in &self.images {
            zip.start_file(Self::image_path(step_id), image_options)?;
            zip.write_all(png)?;
        }

        zip.finish()?;
        Ok(())
    }

    /// Liest einen Container aus einem beliebigen Reader.
    ///
    /// Bilder, die keinem Schritt zugeordnet sind, werden tolerant mitgeladen;
    /// unbekannte JSON-Felder werden ignoriert (Vorwärtskompatibilität
    /// innerhalb derselben Schema-Version).
    pub fn read_from<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut zip = ZipArchive::new(reader)?;
        let guide = read_guide(&mut zip)?;

        let mut images = BTreeMap::new();
        let mut total: u64 = 0;
        let prefix = format!("{IMAGES_DIR}/");
        for index in 0..zip.len() {
            let entry = zip.by_index(index)?;
            let name = entry.name().to_owned();
            if let Some(step_id) = name
                .strip_prefix(&prefix)
                .and_then(|rest| rest.strip_suffix(".png"))
            {
                if !limits::is_valid_step_id(step_id) {
                    continue;
                }
                let png = read_image(entry)?;
                limits::add_image_bytes(&mut total, png.len()).map_err(StoreError::Invalid)?;
                images.insert(step_id.to_owned(), png);
            }
        }

        Ok(Self { guide, images })
    }

    /// Öffnet eine `.steps`-Datei.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::open(path)?;
        Self::read_from(file)
    }
}

/// Liest nur `guide.json` eines Containers, ohne Bilder.
pub fn read_guide_from<R: Read + Seek>(reader: R) -> Result<Guide> {
    read_guide(&mut ZipArchive::new(reader)?)
}

/// Liest nur `guide.json` einer `.steps`-Datei, ohne Bilder.
pub fn open_guide(path: impl AsRef<Path>) -> Result<Guide> {
    read_guide_from(File::open(path)?)
}

/// Guide plus der Screenshot des ersten Schritts, der einen hat. Die übrigen
/// Bilder bleiben ungelesen, damit Vorschauen auch bei großen Dateien schnell sind.
#[derive(Debug, Clone, PartialEq)]
pub struct Preview {
    /// Inhalt von `guide.json`.
    pub guide: Guide,
    /// PNG-Bytes des ersten Schritts mit Screenshot.
    pub first_image: Option<Vec<u8>>,
}

impl Preview {
    /// Liest die Vorschau aus einem beliebigen Reader.
    pub fn read_from<R: Read + Seek>(reader: R) -> Result<Self> {
        let mut zip = ZipArchive::new(reader)?;
        let guide = read_guide(&mut zip)?;
        let mut first_image = None;
        for step in &guide.steps {
            let Ok(entry) = zip.by_name(&StepsFile::image_path(&step.id)) else {
                continue;
            };
            first_image = Some(read_image(entry)?);
            break;
        }
        Ok(Self { guide, first_image })
    }

    /// Liest die Vorschau einer `.steps`-Datei.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::read_from(File::open(path)?)
    }
}

/// Einziger Weg, auf dem `guide.json` gelesen wird: Container-, Größen- und
/// Inhaltsgrenzen aus [`limits`] gelten damit für jede Datei von außen.
fn read_guide<R: Read + Seek>(zip: &mut ZipArchive<R>) -> Result<Guide> {
    if zip.len() > limits::MAX_ZIP_ENTRIES {
        return Err(StoreError::Invalid(format!(
            "more than {} zip entries",
            limits::MAX_ZIP_ENTRIES
        )));
    }
    let entry = zip
        .by_name(GUIDE_FILE_NAME)
        .map_err(|_| StoreError::MissingGuide)?;
    let raw = read_at_most(entry, limits::MAX_GUIDE_JSON_BYTES, GUIDE_FILE_NAME)?;
    parse_guide(&raw)
}

/// Schema-Version, Schema und Inhaltsgrenzen von `guide.json`. Lesen und
/// Schreiben gehen beide hier durch.
fn parse_guide(raw: &[u8]) -> Result<Guide> {
    let value: serde_json::Value = serde_json::from_slice(raw)?;
    let found = value
        .get("schema_version")
        .and_then(serde_json::Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .unwrap_or(0);
    if found == 0 || found > SCHEMA_VERSION {
        return Err(StoreError::UnsupportedSchemaVersion {
            found,
            supported: SCHEMA_VERSION,
        });
    }
    let guide: Guide = serde_json::from_value(value)?;
    limits::check_guide(&guide).map_err(StoreError::Invalid)?;
    Ok(guide)
}

fn read_image(entry: impl Read) -> Result<Vec<u8>> {
    let png = read_at_most(entry, limits::MAX_IMAGE_BYTES, "image")?;
    limits::check_image(&png).map_err(StoreError::Invalid)?;
    Ok(png)
}

/// Liest höchstens `max` Bytes; die angegebene Größe im ZIP zählt nicht,
/// nur was tatsächlich entpackt wird.
fn read_at_most(reader: impl Read, max: u64, what: &str) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(max + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > max {
        return Err(StoreError::Invalid(format!("{what} exceeds {max} bytes")));
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::{ReadFailure, StepsFile, StoreError};

    #[test]
    fn read_failures_name_the_cause() {
        let missing = StepsFile::open(std::env::temp_dir().join("hows-no-such-guide.steps"))
            .expect_err("missing file");
        assert_eq!(missing.read_failure(), ReadFailure::NotFound);
        let junk =
            StepsFile::read_from(std::io::Cursor::new(b"not a zip".to_vec())).expect_err("junk");
        assert_eq!(junk.read_failure(), ReadFailure::Corrupt);
        let newer = StoreError::UnsupportedSchemaVersion {
            found: 2,
            supported: 1,
        };
        assert_eq!(newer.read_failure(), ReadFailure::NewerVersion);
        assert_eq!(
            ReadFailure::ALL.map(ReadFailure::code),
            ["not_found", "corrupt", "newer_version"]
        );
    }
}
