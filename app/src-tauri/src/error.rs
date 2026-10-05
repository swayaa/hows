//! Fehler der Tauri-Commands: ein stabiler Code für die Oberfläche plus ein
//! technisches Detail für Logs.
//!
//! Die Oberfläche zeigt nie `detail`, sondern übersetzt `code` über die
//! Sprachdateien (`errors` in `app/src/locales`). Jeder Code braucht dort
//! einen Text in jeder Sprache; `app/src/commandError.test.ts` prüft das.
//! Die Codes der Lesefehler teilt die App mit `steps_store::ReadFailure` und
//! den `error.*`-Texten der CLI in `core/i18n`.

use std::fmt::Display;

use serde::Serialize;

use steps_store::{ReadFailure, StoreError};

use crate::open_guide::OpenErrorKind;

/// Stabiler Fehlercode; serialisiert in `snake_case`, z. B. `save_failed`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// Die Datei gibt es nicht (mehr).
    NotFound,
    /// Die Datei ist keine `.steps`-Datei.
    NotSteps,
    /// Die `.steps`-Datei ist beschädigt.
    Corrupt,
    /// Die `.steps`-Datei stammt aus einer neueren Version.
    NewerVersion,
    /// Im Editor ist keine Anleitung geöffnet.
    NoGuide,
    /// Die Anleitung ließ sich nicht speichern.
    SaveFailed,
    /// Ein Export ließ sich nicht schreiben.
    ExportFailed,
    /// Ein Zielordner ließ sich nicht anlegen oder finden.
    FolderUnavailable,
    /// Der gewählte Ordner ist nicht nutzbar; `path` nennt den Ersatzordner.
    InvalidFolder,
    /// Die Einstellungsdatei ließ sich nicht schreiben.
    SettingsNotSaved,
    /// Das Tastenkürzel ist ungültig.
    HotkeyInvalid,
    /// Eine andere App belegt das Tastenkürzel.
    HotkeyTaken,
    /// Aufnahme und Pause sollen dasselbe Kürzel bekommen.
    HotkeyConflict,
    /// Der Datei- oder Ordnerdialog ließ sich nicht öffnen.
    DialogFailed,
    /// Der Pfad zum Anzeigen gibt es nicht mehr.
    PathMissing,
    /// Der Dateimanager ließ sich nicht öffnen.
    RevealFailed,
    /// Das Aufnahmefenster ließ sich nicht platzieren.
    WindowFailed,
    /// Der Input-Hook für Maus und Tastatur ließ sich nicht starten; die
    /// Aufnahme läuft nicht.
    InputHookFailed,
    /// Interner Fehler (vergifteter Lock, abgebrochener Thread, falscher Aufruf).
    Internal,
}

/// Antwort eines fehlgeschlagenen Commands an die Oberfläche.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CommandError {
    pub code: ErrorCode,
    /// Technische Ursache, nur für Logs und Fehlerberichte.
    pub detail: String,
    /// Pfad, den die Oberfläche zeigen darf, z. B. der Ersatzordner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

impl CommandError {
    pub fn new(code: ErrorCode, detail: impl Display) -> Self {
        Self {
            code,
            detail: detail.to_string(),
            path: None,
        }
    }

    /// Für `map_err`: `.map_err(CommandError::with(ErrorCode::SaveFailed))`.
    pub fn with<E: Display>(code: ErrorCode) -> impl FnOnce(E) -> Self {
        move |error| Self::new(code, error)
    }

    #[must_use]
    pub fn at_path(mut self, path: impl Display) -> Self {
        self.path = Some(path.to_string());
        self
    }

    /// Vergifteter Lock: `.map_err(CommandError::poisoned("guide"))`.
    pub fn poisoned<E>(what: &'static str) -> impl FnOnce(E) -> Self {
        move |_| Self::new(ErrorCode::Internal, format_args!("{what} lock poisoned"))
    }

    pub fn no_guide() -> Self {
        Self::new(ErrorCode::NoGuide, "no guide in the editor")
    }
}

impl Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.code, self.detail)
    }
}

impl From<OpenErrorKind> for ErrorCode {
    fn from(kind: OpenErrorKind) -> Self {
        match kind {
            OpenErrorKind::NotFound => Self::NotFound,
            OpenErrorKind::NotSteps => Self::NotSteps,
        }
    }
}

impl From<ReadFailure> for ErrorCode {
    fn from(failure: ReadFailure) -> Self {
        match failure {
            ReadFailure::NotFound => Self::NotFound,
            ReadFailure::Corrupt => Self::Corrupt,
            ReadFailure::NewerVersion => Self::NewerVersion,
        }
    }
}

/// Fehler beim Lesen einer `.steps`-Datei; eine neuere Schema-Version bekommt
/// einen eigenen Code, damit die Oberfläche zum Update raten kann.
impl From<StoreError> for CommandError {
    fn from(error: StoreError) -> Self {
        Self::new(error.read_failure().into(), error)
    }
}

#[cfg(test)]
mod tests {
    use super::{CommandError, ErrorCode, ReadFailure, StoreError};

    #[test]
    fn fehler_werden_als_code_und_detail_serialisiert() {
        let error = CommandError::new(ErrorCode::SaveFailed, "Zugriff verweigert");
        assert_eq!(
            serde_json::to_value(&error).unwrap(),
            serde_json::json!({ "code": "save_failed", "detail": "Zugriff verweigert" })
        );
        let folder = CommandError::new(ErrorCode::InvalidFolder, "x").at_path("C:\\Docs");
        assert_eq!(
            serde_json::to_value(&folder).unwrap()["path"],
            serde_json::json!("C:\\Docs")
        );
    }

    #[test]
    fn neuere_schema_version_hat_einen_eigenen_code() {
        let newer = StoreError::UnsupportedSchemaVersion {
            found: 2,
            supported: 1,
        };
        assert_eq!(CommandError::from(newer).code, ErrorCode::NewerVersion);
        assert_eq!(
            CommandError::from(StoreError::MissingGuide).code,
            ErrorCode::Corrupt
        );
    }

    #[test]
    fn lesefehler_teilen_die_codes_mit_dem_kern() {
        for failure in ReadFailure::ALL {
            assert_eq!(
                serde_json::to_value(ErrorCode::from(failure)).unwrap(),
                serde_json::json!(failure.code()),
            );
        }
        let gone = std::io::Error::from(std::io::ErrorKind::NotFound);
        assert_eq!(
            CommandError::from(StoreError::Io(gone)).code,
            ErrorCode::NotFound
        );
    }
}
