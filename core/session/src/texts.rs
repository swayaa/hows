//! Deterministische Schritttexte (Spec §2, Entscheidung 4). Die
//! Formulierungen kommen aus `steps-i18n`; hier steht nur, welcher Name
//! eines Elements in welche Vorlage kommt.

use chrono::{DateTime, FixedOffset, Local, Offset, TimeZone, Utc};
use steps_i18n::{Key, Locale};
use steps_store::{ElementInfo, ScrollDirection, Step};

/// Art des Klicks für die Textgenerierung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickKind {
    /// Einfacher Linksklick.
    Single,
    /// Doppelklick.
    Double,
    /// Rechtsklick.
    Right,
}

/// UIA-Typen, die allein als Schritt-Nomen unbrauchbar sind.
fn is_junk_control_type(value: &str) -> bool {
    [
        "Group",
        "Pane",
        "List",
        "Custom",
        "Document",
        "Thumb",
        "ScrollBar",
    ]
    .iter()
    .any(|junk| value.eq_ignore_ascii_case(junk))
}

fn non_empty(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
}

/// Accessible Name oder ARIA-Label, nie nackte UIA-Junk-Typen.
fn named_control(element: &ElementInfo) -> Option<&str> {
    non_empty(element.name.as_deref())
        .or_else(|| non_empty(element.aria_label.as_deref()))
        .filter(|name| !is_junk_control_type(name))
}

/// Kontext für „in …“: Seitentitel (Browser) vor Fenstertitel, Suffixe weg.
fn context_label(element: &ElementInfo) -> Option<String> {
    non_empty(element.page_title.as_deref())
        .or_else(|| non_empty(element.window_title.as_deref()))
        .map(strip_browser_suffix)
        .filter(|title| !title.is_empty())
}

/// Text für Klick-Schritte. Ohne Namen und ohne Kontext bleibt er leer
/// (nur Screenshot), statt ein Group- oder Button-Nomen zu erfinden.
pub fn click(locale: Locale, kind: ClickKind, element: &ElementInfo) -> String {
    let (both, target_only, context_only) = match kind {
        ClickKind::Single => (
            Key::ClickTargetInContext,
            Key::ClickTarget,
            Key::ClickInContext,
        ),
        ClickKind::Double => (
            Key::DoubleClickTargetInContext,
            Key::DoubleClickTarget,
            Key::DoubleClickInContext,
        ),
        ClickKind::Right => (
            Key::RightClickTargetInContext,
            Key::RightClickTarget,
            Key::RightClickInContext,
        ),
    };
    let target = named_control(element);
    let context = context_label(element);
    match (target, context.as_deref()) {
        (Some(target), Some(context)) => {
            locale.fill(both, &[("target", target), ("context", context)])
        }
        (Some(target), None) => locale.fill(target_only, &[("target", target)]),
        (None, Some(context)) => locale.fill(context_only, &[("context", context)]),
        (None, None) => String::new(),
    }
}

/// Text für einen Tastenkombination-Schritt.
///
/// `canonical_combo` ist die sprachneutrale Form aus den Aufnahme-Daten
/// (z. B. `Ctrl+Shift+S`); Modifier werden lokalisiert dargestellt.
pub fn key_combo(locale: Locale, canonical_combo: &str) -> String {
    let localized: Vec<&str> = canonical_combo
        .split('+')
        .map(|part| match part {
            "Ctrl" => locale.text(Key::ModifierCtrl),
            "Shift" => locale.text(Key::ModifierShift),
            "Alt" => locale.text(Key::ModifierAlt),
            "Win" => locale.text(Key::ModifierWin),
            key => key,
        })
        .collect();
    locale.fill(Key::KeyCombo, &[("combo", &localized.join("+"))])
}

/// Text für einen zusammengefassten Scroll-Schritt.
///
/// Der Name ist das Fenster ([`context_label`]), nicht das Steuerelement
/// unter dem ersten Tick. `amount` ist die Zahl der Rad-Raster.
pub fn scroll(
    locale: Locale,
    direction: ScrollDirection,
    element: &ElementInfo,
    amount: u32,
) -> String {
    let (within, plain) = match direction {
        ScrollDirection::Up => (Key::ScrollUpIn, Key::ScrollUp),
        ScrollDirection::Down => (Key::ScrollDownIn, Key::ScrollDown),
        ScrollDirection::Left => (Key::ScrollLeftIn, Key::ScrollLeft),
        ScrollDirection::Right => (Key::ScrollRightIn, Key::ScrollRight),
    };
    let amount = amount.to_string();
    match context_label(element) {
        Some(target) => locale.fill(within, &[("amount", &amount), ("target", &target)]),
        None => locale.fill(plain, &[("amount", &amount)]),
    }
}

/// Text für einen zusammengefassten Texteingabe-Schritt, nie mit Inhalt.
pub fn text_input(locale: Locale, element: &ElementInfo) -> String {
    match target_label(element) {
        Some(target) => locale.fill(Key::TextInputIn, &[("target", &target)]),
        None => locale.text(Key::TextInput).to_owned(),
    }
}

fn target_label(element: &ElementInfo) -> Option<String> {
    named_control(element)
        .map(str::to_owned)
        .or_else(|| context_label(element))
}

/// Browser-Namen, die als ` - Name` / ` — Name` / ` – Name` aus Fenstertiteln
/// entfernt werden.
const BROWSER_NAMES: &[&str] = &[
    "Google Chrome",
    "Mozilla Firefox",
    "Microsoft Edge",
    "Microsoft\u{200B} Edge",
    "Edge",
    "Chrome",
    "Firefox",
    "Chromium",
    "Brave",
];
const TITLE_DASHES: &[&str] = &[" - ", " \u{2014} ", " \u{2013} "];

fn strip_browser_suffix(title: &str) -> String {
    let mut result = title.trim().to_owned();
    loop {
        let before = result.len();
        for (dash, name) in TITLE_DASHES
            .iter()
            .flat_map(|dash| BROWSER_NAMES.iter().map(move |name| (dash, name)))
        {
            if let Some(stripped) = result
                .strip_suffix(name)
                .and_then(|rest| rest.strip_suffix(dash))
            {
                result = stripped.trim_end().to_owned();
            }
        }
        if result.len() == before {
            break;
        }
    }
    result
}

/// Datum und Uhrzeit einer Aufnahme in Ortszeit, für die Platzhalter
/// `{date}` (`2026-09-30`) und `{time}` (`14.05`) in Titel und Dateinamen.
/// Die Uhrzeit hat einen Punkt, weil Windows keinen Doppelpunkt in
/// Dateinamen erlaubt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stamp {
    /// `YYYY-MM-DD`.
    pub date: String,
    /// `HH.MM`, 24 Stunden.
    pub time: String,
}

impl Stamp {
    /// In der Zeitzone dieses Rechners, mit der Sommerzeit, die zum
    /// Zeitpunkt galt.
    pub fn local(at_ms: u64) -> Self {
        let offset = i64::try_from(at_ms)
            .ok()
            .and_then(|ms| Local.timestamp_millis_opt(ms).single())
            .map_or(0, |time| time.offset().local_minus_utc());
        Self::with_offset(at_ms, offset)
    }

    /// Mit fester Abweichung von UTC in Sekunden, etwa `7200` für MESZ.
    pub fn with_offset(at_ms: u64, offset_seconds: i32) -> Self {
        let offset = FixedOffset::east_opt(offset_seconds).unwrap_or_else(|| Utc.fix());
        let time = i64::try_from(at_ms)
            .ok()
            .and_then(DateTime::from_timestamp_millis)
            .unwrap_or_default()
            .with_timezone(&offset);
        Self {
            date: time.format("%Y-%m-%d").to_string(),
            time: time.format("%H.%M").to_string(),
        }
    }
}

/// Die App der Aufnahme für `{app}`: die des letzten Schritts, der eine kennt.
pub fn app_name(steps: &[Step]) -> Option<String> {
    steps.iter().rev().find_map(|step| {
        step.element
            .as_ref()
            .and_then(|el| non_empty(el.app_name.as_deref()))
            .map(str::to_owned)
    })
}

/// Leitet den Guide-Titel aus App und Datum ab (`guide.title`), sonst leer,
/// dann fragt die UI nach einem Namen.
///
/// Fenster- und Seitentitel fließen bewusst nicht ein: Sie enthalten oft
/// Mail-Betreffs, Kontonamen oder Dokumentnamen, und der Titel ist das
/// Erste, was Empfänger einer geteilten Anleitung sehen.
pub fn derive_guide_title(locale: Locale, steps: &[Step], stamp: &Stamp) -> String {
    guide_title_from(locale.text(Key::GuideTitle), steps, stamp)
}

/// Wie [`derive_guide_title`], mit eigener Vorlage. Platzhalter sind `{app}`,
/// `{date}` und `{time}`; nutzt die Vorlage `{app}` und ist keine App
/// bekannt, bleibt der Titel leer.
pub fn guide_title_from(template: &str, steps: &[Step], stamp: &Stamp) -> String {
    let when = [("date", stamp.date.as_str()), ("time", stamp.time.as_str())];
    match app_name(steps) {
        Some(app) => steps_i18n::fill(template, &[("app", &app), when[0], when[1]]),
        None if steps_i18n::placeholders(template).contains(&"app") => String::new(),
        None => steps_i18n::fill(template, &when),
    }
    .trim()
    .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use steps_store::{Action, StepText};

    /// 2026-09-17 00:00 UTC
    const DAY_MS: u64 = 1_789_603_200_000;
    /// 2026-09-30 22:30 UTC, abends in Deutschland schon der 1. Oktober.
    const EVENING_MS: u64 = 1_790_807_400_000;
    const CEST: i32 = 2 * 3600;

    fn day() -> Stamp {
        Stamp::with_offset(DAY_MS, 0)
    }

    fn en() -> Locale {
        Locale::resolve("en")
    }

    fn de() -> Locale {
        Locale::resolve("de")
    }

    fn element(
        name: Option<&str>,
        control_type: Option<&str>,
        window_title: Option<&str>,
        page_title: Option<&str>,
        aria_label: Option<&str>,
    ) -> ElementInfo {
        ElementInfo {
            name: name.map(str::to_owned),
            control_type: control_type.map(str::to_owned),
            window_title: window_title.map(str::to_owned),
            page_title: page_title.map(str::to_owned),
            aria_label: aria_label.map(str::to_owned),
            ..ElementInfo::default()
        }
    }

    fn single_click(locale: Locale, element: &ElementInfo) -> String {
        click(locale, ClickKind::Single, element)
    }

    fn step_with(element: ElementInfo) -> Step {
        Step {
            id: "s1".into(),
            action: Action::Click,
            timestamp_ms: 1,
            position: None,
            monitor: None,
            element: Some(element),
            text: StepText::generated("x"),
            overlays: Vec::new(),
            crop: None,
        }
    }

    #[test]
    fn generic_control_type_ohne_name_nutzt_fenster() {
        let el = element(None, Some("Group"), Some("Edge"), None, None);
        assert_eq!(single_click(en(), &el), "Click in “Edge”");
        assert_eq!(single_click(de(), &el), "Klicke in „Edge“");
    }

    #[test]
    fn name_gleich_generic_type_wird_ignoriert() {
        let el = element(Some("Pane"), Some("Pane"), Some("Explorer"), None, None);
        assert_eq!(single_click(en(), &el), "Click in “Explorer”");
    }

    #[test]
    fn junk_types_custom_document_scrollbar() {
        for junk in ["Custom", "Document", "ScrollBar", "Thumb"] {
            let el = element(None, Some(junk), Some("Settings – Chrome"), None, None);
            let text = single_click(en(), &el);
            assert_eq!(text, "Click in “Settings”", "junk={junk}");
            assert!(!text.contains(junk));
        }
    }

    #[test]
    fn nameless_ohne_kontext_ist_leerer_text() {
        let el = element(None, Some("Group"), None, None, None);
        assert_eq!(single_click(en(), &el), "");
        assert_eq!(single_click(de(), &el), "");
    }

    #[test]
    fn control_type_allein_wird_nie_nomen() {
        let el = element(None, Some("Button"), Some("Export guide"), None, None);
        assert_eq!(single_click(en(), &el), "Click in “Export guide”");
        assert!(!single_click(en(), &el).contains("Button"));
    }

    #[test]
    fn named_control_pattern() {
        let el = element(
            Some("Save"),
            Some("Button"),
            Some("Export guide"),
            None,
            None,
        );
        assert_eq!(single_click(en(), &el), "Click “Save” in “Export guide”");
        assert_eq!(
            single_click(Locale::resolve("fr"), &el),
            "Cliquez sur «\u{a0}Save\u{a0}» dans «\u{a0}Export guide\u{a0}»"
        );
    }

    #[test]
    fn click_kinds_use_their_own_sentence() {
        let el = element(Some("Datei"), Some("Button"), None, None, None);
        assert_eq!(
            click(de(), ClickKind::Double, &el),
            "Doppelklicke auf „Datei“"
        );
        assert_eq!(
            click(de(), ClickKind::Right, &el),
            "Klicke mit der rechten Maustaste auf „Datei“"
        );
        assert_eq!(click(en(), ClickKind::Right, &el), "Right-click “Datei”");
    }

    #[test]
    fn page_title_hat_vorrang_vor_window() {
        let el = element(
            None,
            Some("Group"),
            Some("Microsoft Edge"),
            Some("Konto erstellen"),
            None,
        );
        assert_eq!(single_click(en(), &el), "Click in “Konto erstellen”");
    }

    #[test]
    fn aria_label_ersetzt_fehlenden_namen() {
        let el = element(None, Some("Button"), Some("App"), None, Some("Speichern"));
        assert_eq!(single_click(de(), &el), "Klicke auf „Speichern“ in „App“");
    }

    #[test]
    fn key_combo_localizes_modifiers() {
        assert_eq!(key_combo(de(), "Ctrl+Shift+S"), "Drücke Strg+Umschalt+S");
        assert_eq!(key_combo(en(), "Ctrl+Alt+Win+K"), "Press Ctrl+Alt+Win+K");
        assert_eq!(
            key_combo(Locale::resolve("es"), "Ctrl+Shift+S"),
            "Pulsa Ctrl+Mayús+S"
        );
    }

    #[test]
    fn scroll_names_the_window_and_the_amount() {
        let control = element(
            Some("Liste"),
            Some("List"),
            Some("Einstellungen"),
            None,
            None,
        );
        assert_eq!(
            scroll(de(), ScrollDirection::Down, &control, 4),
            "Scrolle 4 in „Einstellungen“ nach unten"
        );
        let page = element(
            Some("Speichern"),
            Some("Button"),
            Some("Microsoft Edge"),
            Some("Konto"),
            None,
        );
        assert_eq!(
            scroll(en(), ScrollDirection::Up, &page, 2),
            "Scroll up 2 in “Konto”"
        );
        let named_only = element(Some("Liste"), Some("List"), None, None, None);
        assert_eq!(
            scroll(de(), ScrollDirection::Down, &named_only, 1),
            "Scrolle 1 nach unten"
        );
        let blank = ElementInfo::default();
        assert_eq!(
            scroll(en(), ScrollDirection::Left, &blank, 2),
            "Scroll left 2"
        );
        assert_eq!(text_input(en(), &named_only), "Type your text in “Liste”");
        assert_eq!(text_input(de(), &blank), "Gib deinen Text ein");
    }

    #[test]
    fn guide_title_never_leaks_window_or_page_titles() {
        let mut outlook = element(
            None,
            Some("Group"),
            Some("Posteingang – Max Muster – Outlook"),
            None,
            None,
        );
        outlook.app_name = Some("olk".into());
        let mut browser = element(
            None,
            None,
            Some("Kontoauszug 09/2026 - Google Chrome"),
            Some("Kontoauszug 09/2026"),
            None,
        );
        browser.url = Some("https://bank.example/konto".into());
        browser.app_name = Some("chrome".into());

        let title = derive_guide_title(de(), &[step_with(outlook), step_with(browser)], &day());
        assert_eq!(title, "Anleitung für chrome (2026-09-17)");
        assert!(!title.contains("Posteingang"));
        assert!(!title.contains("Kontoauszug"));
        assert!(!title.contains("bank.example"));
    }

    #[test]
    fn guide_title_first_step_name_is_soft_not_seed() {
        let el = element(Some("Speichern"), Some("Button"), None, None, None);
        let title = derive_guide_title(de(), &[step_with(el)], &day());
        assert_eq!(title, "");
        assert_ne!(title, "Speichern");
    }

    #[test]
    fn guide_title_from_app_and_date() {
        let mut el = element(None, None, Some("Untitled"), None, None);
        el.app_name = Some("Notepad".into());
        assert_eq!(
            derive_guide_title(de(), &[step_with(el.clone())], &day()),
            "Anleitung für Notepad (2026-09-17)"
        );
        assert_eq!(
            derive_guide_title(en(), &[step_with(el)], &day()),
            "Notepad guide (2026-09-17)"
        );
    }

    #[test]
    fn guide_title_empty_prompts_inline_rename() {
        let title = derive_guide_title(en(), &[], &day());
        assert_eq!(title, "");
        assert_ne!(title, "New guide");
        assert_ne!(title, "Neue Anleitung");
        assert!(!title.starts_with("Recording "));
        assert!(!title.starts_with("Aufnahme "));
    }

    #[test]
    fn guide_title_follows_a_custom_template() {
        let mut el = element(None, None, None, None, None);
        el.app_name = Some("Notepad".into());
        let steps = [step_with(el)];
        assert_eq!(
            guide_title_from("{date} {app} how-to", &steps, &day()),
            "2026-09-17 Notepad how-to"
        );
        assert_eq!(guide_title_from("Onboarding", &steps, &day()), "Onboarding");
        assert_eq!(
            guide_title_from("{app} {nope}", &steps, &day()),
            "Notepad {nope}"
        );
    }

    #[test]
    fn stamp_uses_the_local_day_and_a_file_name_safe_time() {
        assert_eq!(
            Stamp::with_offset(EVENING_MS, 0),
            Stamp {
                date: "2026-09-30".into(),
                time: "22.30".into()
            }
        );
        assert_eq!(
            Stamp::with_offset(EVENING_MS, CEST),
            Stamp {
                date: "2026-10-01".into(),
                time: "00.30".into()
            }
        );
        assert_eq!(
            Stamp::with_offset(EVENING_MS, -5 * 3600),
            Stamp {
                date: "2026-09-30".into(),
                time: "17.30".into()
            }
        );
        assert!(!Stamp::with_offset(EVENING_MS, CEST).time.contains(':'));
    }

    #[test]
    fn guide_title_fills_date_and_time_in_local_time() {
        let mut el = element(None, None, None, None, None);
        el.app_name = Some("Notepad".into());
        let evening = Stamp::with_offset(EVENING_MS, CEST);
        assert_eq!(
            guide_title_from("{app} {date} {time}", &[step_with(el)], &evening),
            "Notepad 2026-10-01 00.30"
        );
        assert_eq!(
            derive_guide_title(de(), &[], &evening),
            "",
            "the language template needs an app"
        );
        assert_eq!(guide_title_from("{time}", &[], &evening), "00.30");
    }

    #[test]
    fn guide_title_without_app_is_empty_only_when_the_template_needs_it() {
        assert_eq!(guide_title_from("{app} ({date})", &[], &day()), "");
        assert_eq!(
            guide_title_from("Guide {date}", &[], &day()),
            "Guide 2026-09-17"
        );
    }
}
