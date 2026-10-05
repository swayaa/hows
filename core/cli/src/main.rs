//! Kleines CLI für Tip-/CI-/Agent-Pfade ohne Tauri-GUI.
//!
//! - `steps-cli export <datei.steps> --format html|pdf|markdown|json -o <pfad>
//!   [--brand <code>] [--accent <#hex>] [--no-credit]`
//! - `steps-cli demo -o ./out`: Demo-`.steps` + HTML/MD/JSON (kein Capture)
//! - `steps-cli from-script <script.json> -o guide.steps [--html [guide.html]]`
//!
//! Jeder Befehl nimmt `--lang <code|system>`. Ohne Angabe gilt die
//! Windows-Anzeigesprache, sonst Englisch. Texte kommen aus `steps-i18n`.

mod error;

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use error::{format_names, language_codes, resolve_format, CliError, MODES};
use serde::Deserialize;
use steps_export::{
    brand, write_html_export, write_markdown_export, write_pdf_export, ExportMode, ExportStyle,
    PdfPage,
};
use steps_i18n::{Key, Locale};
use steps_store::{
    unused_dir_path, unused_file_path, Action, ElementInfo, Environment, Guide, Monitor, Position,
    Step, StepText, StepsFile,
};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let (locale, args) = match split_language(&args) {
        Ok(found) => found,
        Err(error) => return fail(Locale::system(), &error),
    };
    match run(locale, &args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => fail(locale, &error),
    }
}

fn fail(locale: Locale, error: &CliError) -> ExitCode {
    eprintln!(
        "{}",
        locale.fill(Key::CliError, &[("message", &error.message(locale))])
    );
    eprintln!();
    eprintln!("{}", usage(locale));
    ExitCode::FAILURE
}

/// Nimmt `--lang <code>` aus den Argumenten, egal an welcher Stelle.
fn split_language(args: &[String]) -> Result<(Locale, Vec<String>), CliError> {
    let mut locale = None;
    let mut rest = Vec::with_capacity(args.len());
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == "--lang" {
            let value = iter.next().ok_or(CliError::MissingValue("--lang"))?;
            locale = Some(parse_language(value)?);
        } else {
            rest.push(arg.clone());
        }
    }
    Ok((locale.unwrap_or_else(Locale::system), rest))
}

fn parse_language(value: &str) -> Result<Locale, CliError> {
    if steps_i18n::is_system(value) {
        return Ok(Locale::system());
    }
    Locale::find(value).ok_or_else(|| CliError::UnknownLanguage(value.to_owned()))
}

fn usage(locale: Locale) -> String {
    let text = |key| locale.text(key);
    let brand = locale.fill(Key::CliOptionBrand, &[("brand", brand::default_brand())]);
    let credit = locale.fill(
        Key::CliOptionNoCredit,
        &[("credit", locale.text(Key::ExportCredit))],
    );
    let lang = locale.fill(Key::CliOptionLang, &[("codes", &language_codes())]);
    format!(
        "{usage}\n  \
         steps-cli export <input.steps> --format {formats} [--mode {modes}] -o <path>\n    \
           [--brand <code>] [--accent <#rrggbb>] [--no-credit]\n  \
         steps-cli demo -o <folder>\n  \
         steps-cli from-script <script.json> -o <out.steps> [--html [<out.html>]]\n\n\
         {formats_heading}\n  \
           html         {html}\n  \
           pdf          {pdf}\n  \
           markdown     {markdown}\n  \
           json         {json}\n\n\
         {options}\n  \
           --brand      {brand}\n  \
           --accent     {accent}\n  \
           --no-credit  {credit}\n  \
           --lang       {lang}\n\n\
         {examples}\n  \
           steps-cli demo -o ./out\n    \
             {demo}\n  \
           steps-cli from-script steps.json -o guide.steps --html\n    \
             {script}",
        usage = text(Key::CliUsage),
        formats = format_names().join("|"),
        modes = MODES.join("|"),
        formats_heading = text(Key::CliFormats),
        html = text(Key::CliFormatHtml),
        pdf = text(Key::CliFormatPdf),
        markdown = text(Key::CliFormatMarkdown),
        json = text(Key::CliFormatJson),
        options = text(Key::CliOptions),
        accent = text(Key::CliOptionAccent),
        examples = text(Key::CliExamples),
        demo = text(Key::CliExampleDemo),
        script = text(Key::CliExampleScript),
    )
}

fn run(locale: Locale, args: &[String]) -> Result<(), CliError> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(CliError::NoCommand);
    };
    match command {
        "export" => export_command(locale, &args[1..]),
        "demo" => demo_command(locale, &args[1..]),
        "from-script" => from_script_command(locale, &args[1..]),
        "help" | "-h" | "--help" => {
            println!("{}", usage(locale));
            Ok(())
        }
        other => Err(CliError::UnknownCommand(other.to_owned())),
    }
}

fn export_command(locale: Locale, args: &[String]) -> Result<(), CliError> {
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    let mut format = "html".to_owned();
    let mut mode = ExportMode::Sop;
    let mut style = ExportStyle::default();

    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--format" | "-f" => {
                next_value(args, &mut index, "--format")?.clone_into(&mut format);
            }
            "--mode" | "-m" => {
                let value = next_value(args, &mut index, "--mode")?;
                mode = parse_mode(value)?;
            }
            "--brand" => {
                let value = next_value(args, &mut index, "--brand")?;
                if brand::find_brand(value).is_none() {
                    return Err(CliError::UnknownBrand(value.to_owned()));
                }
                value.clone_into(&mut style.brand);
            }
            "--accent" => {
                let value = next_value(args, &mut index, "--accent")?;
                if brand::rgb_unit(value).is_none() {
                    return Err(CliError::InvalidAccent(value.to_owned()));
                }
                value.clone_into(&mut style.accent);
            }
            "--no-credit" => {
                style.credit = false;
                index += 1;
            }
            "-o" | "--out" | "--output" => {
                output = Some(PathBuf::from(next_value(args, &mut index, "-o")?));
            }
            flag if flag.starts_with('-') => {
                return Err(CliError::UnknownOption(flag.to_owned()));
            }
            path if input.is_none() => {
                input = Some(PathBuf::from(path));
                index += 1;
            }
            extra => return Err(CliError::UnexpectedArgument(extra.to_owned())),
        }
    }

    let input = input.ok_or(CliError::MissingArgument("<input.steps>"))?;
    let output = output.ok_or(CliError::MissingArgument("-o <path>"))?;
    let file = StepsFile::open(&input).map_err(|error| CliError::open(&input, &error))?;
    write_export(locale, &file, &format, mode, &style, &output).map(drop)
}

fn write_export(
    locale: Locale,
    file: &StepsFile,
    format: &str,
    mode: ExportMode,
    style: &ExportStyle,
    output: &Path,
) -> Result<PathBuf, CliError> {
    let written = match resolve_format(format)? {
        "html" => {
            let output = unused_file_path(output);
            write_html_export(file, mode, style, &output)
                .map_err(CliError::file(Key::ErrorExportFailed, &output))?;
            output
        }
        "pdf" => {
            let output = unused_file_path(output);
            write_pdf_export(file, mode, PdfPage::default(), style, &output)
                .map_err(CliError::file(Key::ErrorExportFailed, &output))?;
            output
        }
        "markdown" => {
            let output = unused_dir_path(output);
            write_markdown_export(file, mode, style, &output)
                .map_err(CliError::file(Key::ErrorExportFailed, &output))?;
            output.join("guide.md")
        }
        "json" => {
            let output = unused_file_path(output);
            ensure_parent_dir(&output)?;
            std::fs::write(&output, steps_export::json(file, mode))
                .map_err(CliError::file(Key::ErrorExportFailed, &output))?;
            output
        }
        other => return Err(CliError::UnknownFormat(other.to_owned())),
    };
    print_wrote(locale, &written);
    Ok(written)
}

/// Speichert `file` als neue `.steps`-Datei unter `path` oder, falls belegt,
/// unter dem nächsten freien Namen.
fn save_new(locale: Locale, file: &StepsFile, path: &Path) -> Result<PathBuf, CliError> {
    let path = unused_file_path(path);
    file.save(&path)
        .map_err(CliError::file(Key::ErrorSaveFailed, &path))?;
    print_wrote(locale, &path);
    Ok(path)
}

fn print_wrote(locale: Locale, path: &Path) {
    println!(
        "{}",
        locale.fill(Key::CliWrote, &[("path", &path.display().to_string())])
    );
}

fn demo_command(locale: Locale, args: &[String]) -> Result<(), CliError> {
    let mut output_dir = PathBuf::from(".");
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "-o" | "--out" | "--output" => {
                output_dir = PathBuf::from(next_value(args, &mut index, "-o")?);
            }
            flag if flag.starts_with('-') => {
                return Err(CliError::UnknownOption(flag.to_owned()));
            }
            extra => return Err(CliError::UnexpectedArgument(extra.to_owned())),
        }
    }

    std::fs::create_dir_all(&output_dir)
        .map_err(CliError::file(Key::ErrorFolderUnavailable, &output_dir))?;
    let steps_path = save_new(
        locale,
        &demo_steps_file(locale),
        &output_dir.join("demo.steps"),
    )?;
    // Roundtrip: .steps öffnen, dann alle Tip-Formate schreiben.
    let loaded =
        StepsFile::open(&steps_path).map_err(|error| CliError::open(&steps_path, &error))?;
    let style = ExportStyle::default();
    let export = |format, path: PathBuf| {
        write_export(locale, &loaded, format, ExportMode::Sop, &style, &path)
    };
    let html_path = export("html", output_dir.join("demo.html"))?;
    export("pdf", output_dir.join("demo.pdf"))?;
    export("markdown", output_dir.join("demo-markdown"))?;
    export("json", output_dir.join("demo.json"))?;
    println!(
        "{}",
        locale.fill(
            Key::CliOpenInBrowser,
            &[("path", &html_path.display().to_string())]
        )
    );
    Ok(())
}

/// Agent-/Skript-Pfad: JSON mit Titel + Schritten wird ein gültiges `.steps` (+ optional HTML).
fn from_script_command(locale: Locale, args: &[String]) -> Result<(), CliError> {
    let mut input: Option<PathBuf> = None;
    let mut output: Option<PathBuf> = None;
    // `Some(None)`: `--html` ohne Pfad, also `<out>.html` neben der `.steps`-Datei.
    let mut html_out: Option<Option<PathBuf>> = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "-o" | "--out" | "--output" => {
                output = Some(PathBuf::from(next_value(args, &mut index, "-o")?));
            }
            "--html" => {
                // Nur ein `.html`/`.htm`-Argument gilt als Wert, sonst würde
                // `--html script.json` das Skript als Ziel verschlucken.
                if let Some(value) = args.get(index + 1).filter(|value| is_html_path(value)) {
                    html_out = Some(Some(PathBuf::from(value)));
                    index += 2;
                } else {
                    html_out = Some(None);
                    index += 1;
                }
            }
            flag if flag.starts_with('-') => {
                return Err(CliError::UnknownOption(flag.to_owned()));
            }
            path if input.is_none() => {
                input = Some(PathBuf::from(path));
                index += 1;
            }
            extra => return Err(CliError::UnexpectedArgument(extra.to_owned())),
        }
    }

    let input = input.ok_or(CliError::MissingArgument("<script.json>"))?;
    let output = output.ok_or(CliError::MissingArgument("-o <out.steps>"))?;
    let script = load_script(&input)?;
    let base_dir = input.parent().unwrap_or_else(|| Path::new("."));
    let file = script_to_steps_file(&script, locale, base_dir)?;
    ensure_parent_dir(&output)?;
    let output = save_new(locale, &file, &output)?;

    if let Some(html_path) = html_out {
        let html_path = html_path.unwrap_or_else(|| output.with_extension("html"));
        let loaded = StepsFile::open(&output).map_err(|error| CliError::open(&output, &error))?;
        write_export(
            locale,
            &loaded,
            "html",
            ExportMode::Sop,
            &ExportStyle::default(),
            &html_path,
        )?;
    }
    Ok(())
}

fn is_html_path(value: &str) -> bool {
    Path::new(value)
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("html") || extension.eq_ignore_ascii_case("htm")
        })
}

#[derive(Debug, Deserialize)]
struct ScriptDoc {
    title: String,
    #[serde(default)]
    description: Option<String>,
    /// Fehlt die Sprache, gilt die der CLI.
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    steps: Vec<ScriptStep>,
}

#[derive(Debug, Deserialize)]
struct ScriptStep {
    text: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    image: Option<String>,
}

fn load_script(path: &Path) -> Result<ScriptDoc, CliError> {
    let raw = std::fs::read_to_string(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            CliError::file(Key::ErrorNotFound, path)(error)
        } else {
            CliError::InvalidScript(error.to_string())
        }
    })?;
    serde_json::from_str(&raw).map_err(|error| CliError::InvalidScript(error.to_string()))
}

fn script_to_steps_file(
    script: &ScriptDoc,
    locale: Locale,
    base_dir: &Path,
) -> Result<StepsFile, CliError> {
    if script.steps.is_empty() {
        return Err(CliError::EmptyScript);
    }
    let created_at_ms = now_ms();
    let language = script
        .language
        .clone()
        .unwrap_or_else(|| locale.code().to_owned());
    let mut guide = Guide::new(script.title.clone(), created_at_ms, language);
    guide.description.clone_from(&script.description);
    guide.environment = Some(Environment {
        os: Some("script".to_owned()),
        os_version: Some("from-script".to_owned()),
    });

    let guide_locale = Locale::resolve(&guide.language);
    let mut file = StepsFile::new(guide);
    for (index, step) in script.steps.iter().enumerate() {
        let id = format!("step-{:04}", index + 1);
        let text = step
            .title
            .as_ref()
            .filter(|title| !title.trim().is_empty())
            .map_or_else(
                || step.text.clone(),
                |title| {
                    guide_locale.fill(
                        Key::ExportLabel,
                        &[("label", title.as_str()), ("value", step.text.as_str())],
                    )
                },
            );
        file.guide.steps.push(Step {
            id: id.clone(),
            action: Action::ManualNote,
            timestamp_ms: created_at_ms + (index as u64 + 1) * 1_000,
            position: None,
            monitor: None,
            element: None,
            text: StepText::generated(text),
            overlays: Vec::new(),
            crop: None,
        });
        if let Some(image_path) = &step.image {
            let resolved = resolve_image_path(base_dir, image_path);
            let bytes = std::fs::read(&resolved).map_err(|error| CliError::Image {
                step: index + 1,
                path: resolved.clone(),
                detail: error.to_string(),
            })?;
            file.images.insert(id, bytes);
        } else {
            file.images.insert(id, PLACEHOLDER_PNG.to_vec());
        }
    }
    Ok(file)
}

fn resolve_image_path(base_dir: &Path, image: &str) -> PathBuf {
    let path = PathBuf::from(image);
    if path.is_absolute() {
        path
    } else {
        base_dir.join(path)
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|duration| u64::try_from(duration.as_millis()).ok())
        .unwrap_or(0)
}

fn next_value<'a>(
    args: &'a [String],
    index: &mut usize,
    flag: &'static str,
) -> Result<&'a str, CliError> {
    let value = args.get(*index + 1).ok_or(CliError::MissingValue(flag))?;
    *index += 2;
    Ok(value.as_str())
}

fn ensure_parent_dir(path: &Path) -> Result<(), CliError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)
                .map_err(CliError::file(Key::ErrorFolderUnavailable, parent))?;
        }
    }
    Ok(())
}

fn parse_mode(value: &str) -> Result<ExportMode, CliError> {
    match value {
        "sop" => Ok(ExportMode::Sop),
        "bug-report" | "bug_report" | "bug" => Ok(ExportMode::BugReport),
        other => Err(CliError::UnknownMode(other.to_owned())),
    }
}

/// 1×1 PNG (transparent), gültig genug für Browser-Anzeige.
const PLACEHOLDER_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

/// Minimaler Demo-Guide in `locale` mit Platzhalter-PNG, ohne OS-Capture.
fn demo_steps_file(locale: Locale) -> StepsFile {
    let mut guide = Guide::new(
        locale.text(Key::CliDemoTitle),
        1_788_307_200_000,
        locale.code(),
    );
    guide.description = Some(locale.text(Key::CliDemoDescription).to_owned());
    guide.environment = Some(Environment {
        os: Some("Demo".to_owned()),
        os_version: Some("fixture".to_owned()),
    });
    guide.steps = vec![
        demo_click_step(
            locale,
            "step-0001",
            1_788_307_260_000,
            (120, 240),
            locale.text(Key::CliDemoThisPc),
            "TreeItem",
        ),
        demo_click_step(
            locale,
            "step-0002",
            1_788_307_320_000,
            (300, 400),
            locale.text(Key::CliDemoMapDrive),
            "MenuItem",
        ),
    ];

    let mut file = StepsFile::new(guide);
    file.images
        .insert("step-0001".to_owned(), PLACEHOLDER_PNG.to_vec());
    file.images
        .insert("step-0002".to_owned(), PLACEHOLDER_PNG.to_vec());
    file
}

fn demo_click_step(
    locale: Locale,
    id: &str,
    timestamp_ms: u64,
    (x, y): (i32, i32),
    name: &str,
    control_type: &str,
) -> Step {
    Step {
        id: id.to_owned(),
        action: Action::Click,
        timestamp_ms,
        position: Some(Position { x, y }),
        monitor: Some(Monitor {
            index: 0,
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
            scale_factor: 1.0,
        }),
        element: Some(ElementInfo {
            name: Some(name.to_owned()),
            control_type: Some(control_type.to_owned()),
            automation_id: None,
            window_title: Some("Explorer".to_owned()),
            app_name: Some("explorer".to_owned()),
            ..ElementInfo::default()
        }),
        text: StepText::generated(locale.fill(Key::ClickTarget, &[("target", name)])),
        overlays: Vec::new(),
        crop: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn de() -> Locale {
        Locale::resolve("de")
    }

    fn strings(args: &[&str]) -> Vec<String> {
        args.iter().map(|&arg| arg.to_owned()).collect()
    }

    #[test]
    fn demo_file_roundtrip_and_html() {
        let dir = std::env::temp_dir().join(format!("steps-cli-demo-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let file = demo_steps_file(de());
        let steps = dir.join("demo.steps");
        file.save(&steps).unwrap();
        let loaded = StepsFile::open(&steps).unwrap();
        assert_eq!(loaded.guide.steps.len(), 2);
        assert_eq!(loaded.guide.language, "de");

        let html_path = dir.join("demo.html");
        write_html_export(
            &loaded,
            ExportMode::Sop,
            &ExportStyle::default(),
            &html_path,
        )
        .unwrap();
        let html = std::fs::read_to_string(&html_path).unwrap();
        assert!(html.contains("Netzlaufwerk verbinden"));
        assert!(html.contains("data:image/png;base64,"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn demo_guide_speaks_every_language() {
        for locale in Locale::all() {
            let file = demo_steps_file(locale);
            assert_eq!(file.guide.title, locale.text(Key::CliDemoTitle));
            assert_eq!(file.guide.language, locale.code());
            let name = locale.text(Key::CliDemoMapDrive);
            assert!(file.guide.steps[1].text.effective().contains(name));
        }
    }

    #[test]
    fn lang_is_taken_from_anywhere_and_checked() {
        let (locale, rest) =
            split_language(&strings(&["demo", "--lang", "fr", "-o", "out"])).unwrap();
        assert_eq!(locale.code(), "fr");
        assert_eq!(rest, strings(&["demo", "-o", "out"]));

        let (locale, _) = split_language(&strings(&["--lang", "PT-br", "help"])).unwrap();
        assert_eq!(locale.code(), "pt");

        let (locale, _) = split_language(&strings(&["--lang", "system", "help"])).unwrap();
        assert_eq!(locale, Locale::system());

        let unknown = split_language(&strings(&["--lang", "xx"])).unwrap_err();
        let message = unknown.message(Locale::EN);
        assert!(message.contains("xx"), "{message}");
        assert!(message.contains("en, de"), "{message}");
        assert!(message.contains("system"), "{message}");

        let missing = split_language(&strings(&["help", "--lang"])).unwrap_err();
        assert!(missing.message(Locale::EN).contains("--lang"));
    }

    #[test]
    fn usage_names_the_brand_default_and_credit_in_every_language() {
        for locale in Locale::all() {
            let text = usage(locale);
            assert!(text.starts_with(locale.text(Key::CliUsage)), "{text}");
            assert!(
                text.contains(brand::default_brand()),
                "{}: {text}",
                locale.code()
            );
            assert!(text.contains(locale.text(Key::ExportCredit)), "{text}");
            assert!(text.contains("--lang"), "{text}");
            assert!(!text.contains('{'), "{}: {text}", locale.code());
        }
        assert!(usage(de()).contains(&format!("(Standard: {})", brand::default_brand())));
    }

    #[test]
    fn every_error_is_translated() {
        let path = Path::new("guide.steps");
        let missing = StepsFile::open(Path::new("does-not-exist.steps")).unwrap_err();
        let errors = [
            CliError::NoCommand,
            CliError::UnknownCommand("x".to_owned()),
            CliError::UnknownOption("--x".to_owned()),
            CliError::UnexpectedArgument("x".to_owned()),
            CliError::MissingValue("-o"),
            CliError::MissingArgument("<input.steps>"),
            CliError::UnknownBrand("x".to_owned()),
            CliError::UnknownFormat("x".to_owned()),
            CliError::UnknownMode("x".to_owned()),
            CliError::UnknownLanguage("x".to_owned()),
            CliError::InvalidAccent("x".to_owned()),
            CliError::InvalidScript("x".to_owned()),
            CliError::EmptyScript,
            CliError::open(path, &missing),
            CliError::file(Key::ErrorExportFailed, path)("disk full"),
            CliError::Image {
                step: 2,
                path: path.to_owned(),
                detail: "x".to_owned(),
            },
        ];
        for locale in Locale::all() {
            for error in &errors {
                let message = error.message(locale);
                assert!(!message.trim().is_empty(), "{error:?}");
                assert!(!message.contains('{'), "{}: {message}", locale.code());
            }
        }
        assert!(CliError::open(path, &missing)
            .message(Locale::EN)
            .starts_with("guide.steps: file not found ("));
        assert!(CliError::open(path, &missing)
            .message(de())
            .contains(de().text(Key::ErrorNotFound)));
    }

    #[test]
    fn export_command_applies_brand_accent_and_credit() {
        let dir = std::env::temp_dir().join(format!("steps-cli-style-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let steps = dir.join("in.steps");
        demo_steps_file(de()).save(&steps).unwrap();
        let other = brand::brands()
            .iter()
            .find(|candidate| candidate.code != brand::default_brand())
            .expect("zweite Marke");

        let html = dir.join("out.html");
        export_command(
            Locale::EN,
            &[
                steps.display().to_string(),
                "--brand".to_owned(),
                other.code.clone(),
                "--accent".to_owned(),
                "#AA3366".to_owned(),
                "--no-credit".to_owned(),
                "-o".to_owned(),
                html.display().to_string(),
            ],
        )
        .unwrap();
        let body = std::fs::read_to_string(&html).unwrap();
        let palette = brand::brand_palette(&other.code, brand::Skin::Light, "#AA3366");
        assert!(body.contains(&format!("--sl-accent:{};", palette.accent)));
        assert!(body.contains(&format!("--sl-bg:{};", palette.bg)));
        assert!(!body.contains("class=\"credit\""));

        for (flag, value) in [("--brand", "nope"), ("--accent", "rot")] {
            let error = export_command(
                Locale::EN,
                &[
                    steps.display().to_string(),
                    flag.to_owned(),
                    value.to_owned(),
                    "-o".to_owned(),
                    html.display().to_string(),
                ],
            )
            .unwrap_err();
            let message = error.message(Locale::EN);
            assert!(message.contains(value), "{message}");
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn parse_mode_accepts_aliases() {
        assert_eq!(parse_mode("sop").unwrap(), ExportMode::Sop);
        assert_eq!(parse_mode("bug").unwrap(), ExportMode::BugReport);
    }

    #[test]
    fn export_command_writes_html_markdown_json() {
        let dir = std::env::temp_dir().join(format!("steps-cli-export-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let steps = dir.join("in.steps");
        demo_steps_file(de()).save(&steps).unwrap();
        let export = |format: &str, output: &Path, extra: &[&str]| {
            let mut args = vec![
                steps.display().to_string(),
                "--format".to_owned(),
                format.to_owned(),
            ];
            args.extend(strings(extra));
            args.extend(["-o".to_owned(), output.display().to_string()]);
            export_command(Locale::EN, &args).unwrap();
        };

        let html = dir.join("out.html");
        export("html", &html, &[]);
        assert!(html.is_file());

        let pdf = dir.join("out.pdf");
        export("pdf", &pdf, &["--mode", "bug-report"]);
        assert!(std::fs::read(&pdf).unwrap().starts_with(b"%PDF-"));

        let md_dir = dir.join("md-out");
        export("markdown", &md_dir, &[]);
        assert!(md_dir.join("guide.md").is_file());

        let json = dir.join("out.json");
        export("json", &json, &[]);
        let body = std::fs::read_to_string(&json).unwrap();
        assert!(body.contains("Netzlaufwerk verbinden"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn export_never_replaces_an_existing_file() {
        let dir = std::env::temp_dir().join(format!("steps-cli-taken-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let steps = dir.join("in.steps");
        demo_steps_file(de()).save(&steps).unwrap();
        let json = dir.join("out.json");
        std::fs::write(&json, "fremd").unwrap();

        export_command(
            Locale::EN,
            &strings(&[
                &steps.display().to_string(),
                "--format",
                "json",
                "-o",
                &json.display().to_string(),
            ]),
        )
        .unwrap();

        assert_eq!(std::fs::read_to_string(&json).unwrap(), "fremd");
        let second = std::fs::read_to_string(dir.join("out (2).json")).unwrap();
        assert!(second.contains("Netzlaufwerk verbinden"), "{second}");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn markdown_export_into_a_taken_folder_leaves_no_stale_images() {
        let dir = std::env::temp_dir().join(format!("steps-cli-md-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let md_dir = dir.join("md-out");
        let export = |file: &StepsFile| {
            write_export(
                Locale::EN,
                file,
                "markdown",
                ExportMode::Sop,
                &ExportStyle::default(),
                &md_dir,
            )
            .unwrap()
        };
        let image_names = |folder: &Path| {
            let mut names: Vec<String> = std::fs::read_dir(folder.join("images"))
                .unwrap()
                .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
                .collect();
            names.sort();
            names
        };

        let mut file = demo_steps_file(de());
        assert_eq!(export(&file), md_dir.join("guide.md"));
        file.guide.steps.retain(|step| step.id != "step-0002");
        file.images.remove("step-0002");
        let second = export(&file);

        let second_dir = dir.join("md-out (2)");
        assert_eq!(second, second_dir.join("guide.md"));
        assert_eq!(image_names(&second_dir), ["step-0001.png"]);
        assert_eq!(image_names(&md_dir), ["step-0001.png", "step-0002.png"]);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn from_script_writes_steps_and_html() {
        let dir = std::env::temp_dir().join(format!("steps-cli-script-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let script_path = dir.join("script.json");
        std::fs::write(
            &script_path,
            r#"{
              "title": "Agent-Demo",
              "language": "de",
              "steps": [
                {"text": "Erster Schritt"},
                {"text": "Zweiter Schritt", "title": "Hinweis"}
              ]
            }"#,
        )
        .unwrap();

        let steps_out = dir.join("agent.steps");
        let html_out = dir.join("agent.html");
        from_script_command(
            Locale::EN,
            &[
                script_path.display().to_string(),
                "-o".to_owned(),
                steps_out.display().to_string(),
                "--html".to_owned(),
                html_out.display().to_string(),
            ],
        )
        .unwrap();

        let loaded = StepsFile::open(&steps_out).unwrap();
        assert_eq!(loaded.guide.title, "Agent-Demo");
        assert_eq!(loaded.guide.language, "de");
        assert_eq!(loaded.guide.steps.len(), 2);
        assert_eq!(
            loaded.guide.steps[1].text.effective(),
            "Hinweis: Zweiter Schritt"
        );
        assert!(html_out.is_file());
        let html = std::fs::read_to_string(&html_out).unwrap();
        assert!(html.contains("Agent-Demo"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn every_listed_format_is_accepted_and_md_means_markdown() {
        assert_eq!(resolve_format("md").unwrap(), "markdown");
        for name in format_names() {
            assert!(resolve_format(name).is_ok(), "{name}");
            assert!(usage(Locale::EN).contains(name), "{name}");
        }
        let unknown = resolve_format("docx").unwrap_err();
        assert!(unknown.message(Locale::EN).contains("md"));
    }

    #[test]
    fn from_script_labels_steps_in_the_guide_language() {
        let script: ScriptDoc = serde_json::from_str(
            r#"{"title": "Démo", "language": "fr",
                "steps": [{"text": "Clique sur OK", "title": "Note"}]}"#,
        )
        .unwrap();
        let file = script_to_steps_file(&script, Locale::EN, Path::new(".")).unwrap();
        assert_eq!(
            file.guide.steps[0].text.effective(),
            "Note\u{a0}: Clique sur OK"
        );
    }

    #[test]
    fn from_script_html_without_path_lands_next_to_steps() {
        let dir = std::env::temp_dir().join(format!("steps-cli-html-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let script_path = dir.join("script.json");
        std::fs::write(
            &script_path,
            r#"{"title": "Ohne Pfad", "steps": [{"text": "Eins"}]}"#,
        )
        .unwrap();

        // `--html` vor dem Skript: das Skript darf nicht als HTML-Ziel gelten.
        let steps_out = dir.join("guide.steps");
        from_script_command(
            Locale::resolve("fr"),
            &[
                "--html".to_owned(),
                script_path.display().to_string(),
                "-o".to_owned(),
                steps_out.display().to_string(),
            ],
        )
        .unwrap();

        assert!(steps_out.is_file());
        let loaded = StepsFile::open(&steps_out).unwrap();
        assert_eq!(
            loaded.guide.language, "fr",
            "ohne Angabe gilt die CLI-Sprache"
        );
        let html = std::fs::read_to_string(dir.join("guide.html")).unwrap();
        assert!(html.contains("Ohne Pfad"));
        assert!(std::fs::read_to_string(&script_path)
            .unwrap()
            .contains("Ohne Pfad"));

        let first = std::fs::read(&steps_out).unwrap();
        from_script_command(
            Locale::resolve("fr"),
            &strings(&[
                &script_path.display().to_string(),
                "-o",
                &steps_out.display().to_string(),
                "--html",
            ]),
        )
        .unwrap();
        assert_eq!(std::fs::read(&steps_out).unwrap(), first);
        assert!(StepsFile::open(dir.join("guide (2).steps")).is_ok());
        assert!(std::fs::read_to_string(dir.join("guide (2).html"))
            .unwrap()
            .contains("Ohne Pfad"));

        let _ = std::fs::remove_dir_all(&dir);
    }
}
