//! Fehler der CLI als Daten. Der Text entsteht erst beim Ausgeben, in der
//! Sprache von `--lang` oder von Windows.

use std::fmt::Display;
use std::path::{Path, PathBuf};

use steps_export::brand;
use steps_i18n::{Key, Locale};
use steps_store::{ReadFailure, StoreError};

/// Namen von `export --format` und das Format, für das sie stehen; `md` ist
/// die Kurzform von `markdown`.
pub const FORMATS: &[(&str, &str)] = &[
    ("html", "html"),
    ("pdf", "pdf"),
    ("markdown", "markdown"),
    ("md", "markdown"),
    ("json", "json"),
];

/// Alle Namen, die `--format` annimmt.
pub fn format_names() -> Vec<&'static str> {
    FORMATS.iter().map(|(name, _)| *name).collect()
}

/// Format zu einem Namen oder einer Kurzform.
pub fn resolve_format(name: &str) -> Result<&'static str, CliError> {
    FORMATS
        .iter()
        .find(|(known, _)| *known == name)
        .map(|(_, format)| *format)
        .ok_or_else(|| CliError::UnknownFormat(name.to_owned()))
}
/// Modi von `export --mode`.
pub const MODES: &[&str] = &["sop", "bug-report"];

/// Ein Fehler, der den Aufruf beendet.
#[derive(Debug)]
pub enum CliError {
    NoCommand,
    UnknownCommand(String),
    UnknownOption(String),
    UnexpectedArgument(String),
    /// Nach der Option fehlt ihr Wert.
    MissingValue(&'static str),
    /// Ein Pflichtargument wie `<input.steps>` fehlt.
    MissingArgument(&'static str),
    UnknownBrand(String),
    UnknownFormat(String),
    UnknownMode(String),
    UnknownLanguage(String),
    InvalidAccent(String),
    InvalidScript(String),
    EmptyScript,
    /// Eine Datei ließ sich nicht lesen oder schreiben; `key` ist ein
    /// `error.*`-Text, `detail` die technische Ursache.
    File {
        path: PathBuf,
        key: Key,
        detail: String,
    },
    /// Das Bild eines Skript-Schritts ist nicht lesbar.
    Image {
        step: usize,
        path: PathBuf,
        detail: String,
    },
}

impl CliError {
    /// Eine `.steps`-Datei ließ sich nicht öffnen.
    pub fn open(path: &Path, error: &StoreError) -> Self {
        let key = match error.read_failure() {
            ReadFailure::NotFound => Key::ErrorNotFound,
            ReadFailure::Corrupt => Key::ErrorCorrupt,
            ReadFailure::NewerVersion => Key::ErrorNewerVersion,
        };
        Self::File {
            path: path.to_owned(),
            key,
            detail: error.to_string(),
        }
    }

    /// Für `map_err`: `.map_err(CliError::file(Key::ErrorExportFailed, path))`.
    pub fn file<E: Display>(key: Key, path: &Path) -> impl FnOnce(E) -> Self + '_ {
        move |error| Self::File {
            path: path.to_owned(),
            key,
            detail: error.to_string(),
        }
    }

    /// Die Meldung in `locale`.
    pub fn message(&self, locale: Locale) -> String {
        let with_value = |key, value: &str| locale.fill(key, &[("value", value)]);
        let with_known =
            |key, value: &str, known: &str| locale.fill(key, &[("value", value), ("known", known)]);
        match self {
            Self::NoCommand => locale.text(Key::CliNoCommand).to_owned(),
            Self::UnknownCommand(value) => with_value(Key::CliUnknownCommand, value),
            Self::UnknownOption(value) => with_value(Key::CliUnknownOption, value),
            Self::UnexpectedArgument(value) => with_value(Key::CliUnexpectedArgument, value),
            Self::MissingValue(flag) => locale.fill(Key::CliMissingValue, &[("flag", flag)]),
            Self::MissingArgument(argument) => {
                locale.fill(Key::CliMissingArgument, &[("argument", argument)])
            }
            Self::UnknownBrand(value) => with_known(Key::CliUnknownBrand, value, &brand_codes()),
            Self::UnknownFormat(value) => {
                with_known(Key::CliUnknownFormat, value, &format_names().join(", "))
            }
            Self::UnknownMode(value) => with_known(Key::CliUnknownMode, value, &MODES.join(", ")),
            Self::UnknownLanguage(value) => {
                with_known(Key::CliUnknownLanguage, value, &language_codes())
            }
            Self::InvalidAccent(value) => with_value(Key::CliInvalidAccent, value),
            Self::InvalidScript(detail) => {
                locale.fill(Key::CliInvalidScript, &[("detail", detail)])
            }
            Self::EmptyScript => locale.text(Key::CliEmptyScript).to_owned(),
            Self::File { path, key, detail } => {
                format!("{}: {} ({detail})", path.display(), locale.text(*key))
            }
            Self::Image { step, path, detail } => format!(
                "{}: {} ({detail})",
                path.display(),
                locale.fill(Key::CliImageUnreadable, &[("step", &step.to_string())])
            ),
        }
    }
}

/// Codes der Stile aus `brands.json`.
pub fn brand_codes() -> String {
    brand::brands()
        .iter()
        .map(|found| found.code.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Werte von `--lang`: jede Sprache von `steps-i18n` und `system`.
pub fn language_codes() -> String {
    Locale::all()
        .map(Locale::code)
        .chain([steps_i18n::SYSTEM])
        .collect::<Vec<_>>()
        .join(", ")
}
