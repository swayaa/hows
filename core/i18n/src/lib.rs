//! Übersetzungen für alles, was der Kern selbst formuliert: Schritttexte,
//! den vorgeschlagenen Titel, die Beschriftungen der Exporte und die Texte
//! von `steps-cli`.
//!
//! Die Schlüssel `error.<code>` tragen dieselben Codes wie `ErrorCode` der
//! App (`app/src-tauri/src/error.rs`); `app/src/commandError.test.ts` prüft das.
//!
//! Jede Sprache ist eine JSON-Datei unter `locales/`. Eine Sprache kommt
//! hinzu, indem man die Datei anlegt und sie in [`LOCALES`] einträgt; die
//! Tests prüfen, dass sie jeden Schlüssel mit denselben Platzhaltern hat.

use std::collections::HashMap;
use std::sync::OnceLock;

/// Eine unterstützte Sprache.
#[derive(Debug)]
pub struct LocaleInfo {
    /// Sprachcode (ISO 639-1), zugleich `guide.language`.
    pub code: &'static str,
    source: &'static str,
}

/// Alle Sprachen. Englisch steht vorn und ist der Rückfall.
pub const LOCALES: &[LocaleInfo] = &[
    LocaleInfo {
        code: "en",
        source: include_str!("../locales/en.json"),
    },
    LocaleInfo {
        code: "de",
        source: include_str!("../locales/de.json"),
    },
    LocaleInfo {
        code: "fr",
        source: include_str!("../locales/fr.json"),
    },
    LocaleInfo {
        code: "es",
        source: include_str!("../locales/es.json"),
    },
    LocaleInfo {
        code: "it",
        source: include_str!("../locales/it.json"),
    },
    LocaleInfo {
        code: "pt",
        source: include_str!("../locales/pt.json"),
    },
    LocaleInfo {
        code: "nl",
        source: include_str!("../locales/nl.json"),
    },
];

macro_rules! keys {
    ($($variant:ident = $id:literal,)*) => {
        /// Schlüssel eines übersetzten Texts.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Key {
            $(#[doc = concat!("`", $id, "`")] $variant,)*
        }

        impl Key {
            /// Alle Schlüssel.
            pub const ALL: &'static [Key] = &[$(Key::$variant,)*];

            /// Name des Schlüssels in den Sprachdateien.
            pub fn id(self) -> &'static str {
                match self {
                    $(Key::$variant => $id,)*
                }
            }
        }
    };
}

keys! {
    LocaleName = "locale.name",
    ClickTargetInContext = "step.click.target_in_context",
    ClickTarget = "step.click.target",
    ClickInContext = "step.click.in_context",
    DoubleClickTargetInContext = "step.double_click.target_in_context",
    DoubleClickTarget = "step.double_click.target",
    DoubleClickInContext = "step.double_click.in_context",
    RightClickTargetInContext = "step.right_click.target_in_context",
    RightClickTarget = "step.right_click.target",
    RightClickInContext = "step.right_click.in_context",
    KeyCombo = "step.key_combo",
    ScrollUpIn = "step.scroll_up.in",
    ScrollUp = "step.scroll_up",
    ScrollDownIn = "step.scroll_down.in",
    ScrollDown = "step.scroll_down",
    ScrollLeftIn = "step.scroll_left.in",
    ScrollLeft = "step.scroll_left",
    ScrollRightIn = "step.scroll_right.in",
    ScrollRight = "step.scroll_right",
    TextInputIn = "step.text_input.in",
    TextInput = "step.text_input",
    ModifierCtrl = "key.ctrl",
    ModifierShift = "key.shift",
    ModifierAlt = "key.alt",
    ModifierWin = "key.win",
    GuideTitle = "guide.title",
    ExportCreated = "export.created",
    ExportStep = "export.step",
    ExportEnvironment = "export.environment",
    ExportApp = "export.app",
    ExportWindow = "export.window",
    ExportElement = "export.element",
    ExportTime = "export.time",
    ExportPosition = "export.position",
    ExportMonitor = "export.monitor",
    ExportScale = "export.scale",
    ExportUntitled = "export.untitled",
    ExportCredit = "export.credit",
    ExportLabel = "export.label",
    TrayToggle = "tray.toggle",
    TrayPause = "tray.pause",
    TrayShow = "tray.show",
    TrayQuit = "tray.quit",
    DialogOpenGuide = "dialog.open_guide",
    DialogFilterSteps = "dialog.filter_steps",
    DialogFilterAll = "dialog.filter_all",
    DialogChooseFolder = "dialog.choose_folder",
    ErrorNotFound = "error.not_found",
    ErrorCorrupt = "error.corrupt",
    ErrorNewerVersion = "error.newer_version",
    ErrorSaveFailed = "error.save_failed",
    ErrorExportFailed = "error.export_failed",
    ErrorFolderUnavailable = "error.folder_unavailable",
    CliUsage = "cli.usage",
    CliFormats = "cli.formats",
    CliFormatHtml = "cli.format.html",
    CliFormatPdf = "cli.format.pdf",
    CliFormatMarkdown = "cli.format.markdown",
    CliFormatJson = "cli.format.json",
    CliOptions = "cli.options",
    CliOptionBrand = "cli.option.brand",
    CliOptionAccent = "cli.option.accent",
    CliOptionNoCredit = "cli.option.no_credit",
    CliOptionLang = "cli.option.lang",
    CliExamples = "cli.examples",
    CliExampleDemo = "cli.example.demo",
    CliExampleScript = "cli.example.script",
    CliWrote = "cli.wrote",
    CliOpenInBrowser = "cli.open_in_browser",
    CliError = "cli.error",
    CliNoCommand = "cli.error.no_command",
    CliUnknownCommand = "cli.error.unknown_command",
    CliUnknownOption = "cli.error.unknown_option",
    CliUnexpectedArgument = "cli.error.unexpected_argument",
    CliMissingValue = "cli.error.missing_value",
    CliMissingArgument = "cli.error.missing_argument",
    CliUnknownBrand = "cli.error.unknown_brand",
    CliUnknownFormat = "cli.error.unknown_format",
    CliUnknownMode = "cli.error.unknown_mode",
    CliUnknownLanguage = "cli.error.unknown_language",
    CliInvalidAccent = "cli.error.invalid_accent",
    CliInvalidScript = "cli.error.invalid_script",
    CliEmptyScript = "cli.error.empty_script",
    CliImageUnreadable = "cli.error.image_unreadable",
    CliDemoTitle = "cli.demo.title",
    CliDemoDescription = "cli.demo.description",
    CliDemoThisPc = "cli.demo.this_pc",
    CliDemoMapDrive = "cli.demo.map_drive",
}

/// Einstellungswert für „Sprache wie Windows“.
pub const SYSTEM: &str = "system";

type Catalog = HashMap<String, String>;

fn catalogs() -> &'static [Catalog] {
    static CATALOGS: OnceLock<Vec<Catalog>> = OnceLock::new();
    CATALOGS.get_or_init(|| {
        LOCALES
            .iter()
            .map(|info| serde_json::from_str(info.source).unwrap_or_default())
            .collect()
    })
}

/// Eine aufgelöste Sprache aus [`LOCALES`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Locale {
    index: usize,
}

impl Locale {
    /// Englisch.
    pub const EN: Locale = Locale { index: 0 };

    /// Findet die Sprache zu einem Code wie `de`, `de-AT` oder `pt_BR`.
    pub fn find(code: &str) -> Option<Locale> {
        let primary = code
            .trim()
            .split(['-', '_', '.'])
            .next()
            .unwrap_or_default();
        LOCALES
            .iter()
            .position(|info| info.code.eq_ignore_ascii_case(primary))
            .map(|index| Locale { index })
    }

    /// Wie [`Locale::find`], unbekannte Codes ergeben Englisch.
    pub fn resolve(code: &str) -> Locale {
        Self::find(code).unwrap_or(Self::EN)
    }

    /// Anzeigesprache von Windows (`GetUserPreferredUILanguages`), falls
    /// Hows sie kann, sonst Englisch.
    pub fn system() -> Locale {
        sys_locale::get_locale().map_or(Self::EN, |code| Self::resolve(&code))
    }

    /// Sprache zu einem Einstellungswert: [`SYSTEM`] folgt Windows, sonst wie
    /// [`Locale::resolve`].
    pub fn from_setting(value: &str) -> Locale {
        if is_system(value) {
            Self::system()
        } else {
            Self::resolve(value)
        }
    }

    /// Alle Sprachen in der Reihenfolge von [`LOCALES`].
    pub fn all() -> impl Iterator<Item = Locale> {
        (0..LOCALES.len()).map(|index| Locale { index })
    }

    /// Sprachcode, etwa `de`.
    pub fn code(self) -> &'static str {
        LOCALES[self.index].code
    }

    /// Eigenname, etwa `Deutsch`.
    pub fn name(self) -> &'static str {
        self.text(Key::LocaleName)
    }

    /// Der Text zu `key`; fehlt er, der englische.
    pub fn text(self, key: Key) -> &'static str {
        let catalogs = catalogs();
        catalogs[self.index]
            .get(key.id())
            .or_else(|| catalogs[Self::EN.index].get(key.id()))
            .map_or_else(|| key.id(), String::as_str)
    }

    /// Der Text zu `key` mit eingesetzten Platzhaltern, siehe [`fill`].
    pub fn fill(self, key: Key, args: &[(&str, &str)]) -> String {
        fill(self.text(key), args)
    }
}

/// `true` für den Einstellungswert [`SYSTEM`], ohne Rücksicht auf Groß- und
/// Kleinschreibung.
pub fn is_system(value: &str) -> bool {
    value.trim().eq_ignore_ascii_case(SYSTEM)
}

/// Setzt `{name}`-Platzhalter in einem Durchgang ein. Eingesetzte Werte
/// werden nicht erneut ersetzt; unbekannte Platzhalter bleiben stehen.
pub fn fill(template: &str, args: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else {
            out.push_str(&rest[open..]);
            return out;
        };
        let name = &after[..close];
        match args.iter().find(|(arg, _)| *arg == name) {
            Some((_, value)) => out.push_str(value),
            None => out.push_str(&rest[open..=open + close + 1]),
        }
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    out
}

/// Die Platzhalternamen einer Vorlage in Reihenfolge.
pub fn placeholders(template: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else { break };
        names.push(&after[..close]);
        rest = &after[close + 1..];
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn parsed(info: &LocaleInfo) -> Catalog {
        serde_json::from_str(info.source)
            .unwrap_or_else(|error| panic!("{}.json: {error}", info.code))
    }

    #[test]
    fn every_locale_has_exactly_the_known_keys() {
        let expected: BTreeSet<&str> = Key::ALL.iter().map(|key| key.id()).collect();
        for info in LOCALES {
            let catalog = parsed(info);
            let actual: BTreeSet<&str> = catalog.keys().map(String::as_str).collect();
            assert_eq!(actual, expected, "{}.json", info.code);
        }
    }

    #[test]
    fn placeholders_match_english() {
        let english = parsed(&LOCALES[0]);
        for info in &LOCALES[1..] {
            let catalog = parsed(info);
            for key in Key::ALL {
                let mut want = placeholders(&english[key.id()]);
                let mut got = placeholders(&catalog[key.id()]);
                want.sort_unstable();
                got.sort_unstable();
                assert_eq!(got, want, "{}.json {}", info.code, key.id());
            }
        }
    }

    /// Begriffe aus dem Code, die Nutzer nicht kennen; wie `copyLint.test.ts`.
    const INTERNAL_TERMS: [&str; 9] = [
        "pill",
        "primary",
        "soft ledger",
        "sop",
        "byo",
        "chrome",
        "filmstrip",
        "fallback",
        "prefs",
    ];

    /// Verstöße gegen die Textregeln der App: Gedankenstriche, Textpfeile
    /// (U+2190 bis U+21FF, `->`, `<-`) und interne Begriffe als ganzes Wort.
    fn slop(text: &str) -> Vec<&'static str> {
        let mut found = Vec::new();
        if text.contains(['\u{2013}', '\u{2014}']) {
            found.push("dash");
        }
        if text.chars().any(|c| ('\u{2190}'..='\u{21FF}').contains(&c))
            || text.contains("->")
            || text.contains("<-")
        {
            found.push("text arrow");
        }
        let lower = text.to_lowercase();
        let words: Vec<&str> = lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|word| !word.is_empty())
            .collect();
        let has_term = INTERNAL_TERMS.iter().any(|term| {
            let parts: Vec<&str> = term.split(' ').collect();
            words
                .windows(parts.len())
                .any(|window| window == parts.as_slice())
        });
        if has_term {
            found.push("internal term");
        }
        found
    }

    #[test]
    fn texts_have_no_dashes_arrows_internal_terms_or_blanks() {
        for info in LOCALES {
            for (key, text) in parsed(info) {
                assert!(!text.trim().is_empty(), "{}.json {key} is empty", info.code);
                assert_eq!(
                    slop(&text),
                    Vec::<&str>::new(),
                    "{}.json {key}: {text}",
                    info.code
                );
            }
        }
    }

    #[test]
    fn slop_notices_every_rule() {
        assert_eq!(slop("Weiter \u{2014} fertig"), ["dash"]);
        for arrow in ["\u{2190}", "\u{2192}", "\u{21D2}", "\u{21FF}", "->", "<-"] {
            assert_eq!(slop(&format!("A {arrow} B")), ["text arrow"], "{arrow}");
        }
        for term in ["Pill", "SOP", "soft  ledger", "Fallback-Text"] {
            assert_eq!(
                slop(&format!("Die {term} hier")),
                ["internal term"],
                "{term}"
            );
        }
        for fine in [
            "Primärfarbe",
            "Chromebook",
            "Sopran",
            "Schritt 1 von 3",
            "a > b",
        ] {
            assert_eq!(slop(fine), Vec::<&str>::new(), "{fine}");
        }
    }

    #[test]
    fn locale_files_on_disk_are_all_registered() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("locales");
        let mut on_disk: Vec<String> = std::fs::read_dir(dir)
            .expect("locales dir")
            .filter_map(|entry| {
                let name = entry.ok()?.file_name().into_string().ok()?;
                name.strip_suffix(".json").map(str::to_owned)
            })
            .collect();
        on_disk.sort();
        let mut registered: Vec<String> = LOCALES.iter().map(|info| info.code.to_owned()).collect();
        registered.sort();
        assert_eq!(on_disk, registered);
    }

    #[test]
    fn find_uses_the_primary_subtag() {
        assert_eq!(Locale::find("de-AT").map(Locale::code), Some("de"));
        assert_eq!(Locale::find("pt_BR.UTF-8").map(Locale::code), Some("pt"));
        assert_eq!(Locale::find("NL").map(Locale::code), Some("nl"));
        assert_eq!(Locale::find("ja-JP"), None);
        assert_eq!(Locale::resolve("ja-JP").code(), "en");
    }

    #[test]
    fn fill_replaces_once_and_keeps_unknown_placeholders() {
        assert_eq!(
            fill(
                "{app} ({date})",
                &[("app", "{date}"), ("date", "2026-09-30")]
            ),
            "{date} (2026-09-30)"
        );
        assert_eq!(fill("a {missing} b", &[]), "a {missing} b");
        assert_eq!(fill("open {brace", &[]), "open {brace");
        assert_eq!(placeholders("{verb} “{target}”"), vec!["verb", "target"]);
    }

    #[test]
    fn text_falls_back_to_english_names() {
        let german = Locale::resolve("de");
        assert_eq!(german.text(Key::ModifierCtrl), "Strg");
        assert_eq!(german.name(), "Deutsch");
        assert_eq!(Locale::all().count(), LOCALES.len());
    }

    #[test]
    fn system_setting_follows_windows_and_codes_stay_fixed() {
        assert!(is_system(" System "));
        assert!(!is_system("de"));
        assert_eq!(Locale::from_setting("system"), Locale::system());
        assert_eq!(Locale::from_setting("de").code(), "de");
        assert_eq!(Locale::from_setting("klingon"), Locale::EN);
        assert!(LOCALES.iter().all(|info| info.code != SYSTEM));
    }
}
