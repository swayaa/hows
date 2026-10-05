//! Tauri-Commands: Aufnahme-Steuerung, Editor-Operationen, Einstellungen.

use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, MutexGuard};

use serde::Serialize;
use steps_i18n::{Key, Locale};
use steps_session::Stamp;
use steps_store::{
    unused_dir_path, unused_file_path, Action, Crop, Guide, Overlay, Preview, StepsFile,
};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::error::{CommandError, ErrorCode};
use crate::open_guide::{classify_open_path, prune_recent, push_recent, OpenErrorKind};
use crate::recorder::{Command, RecorderConfig};
use crate::settings::{self, Settings};
use crate::tray::TrayMenu;

/// Geteilter App-Zustand.
pub struct AppState {
    /// Kommando-Sender zum Recorder-Thread.
    pub recorder: Sender<Command>,
    /// Zuletzt gestoppte Aufnahme (Editor-Arbeitskopie).
    pub guide: Arc<Mutex<Option<StepsFile>>>,
    /// Aktuelle Einstellungen.
    pub settings: Mutex<Settings>,
    /// `.steps`-Pfad eines zweiten Starts, den das Frontend einmal abholt
    /// und vorher bestätigt.
    pub pending_open: Mutex<Option<String>>,
    /// Hotkey-Slots (`record` | `pause`), deren Registrierung beim Start
    /// scheiterte. Ein Event aus `setup` käme vor dem Frontend-Listener an
    /// und ginge verloren, deshalb liest das Frontend das über `get_settings`.
    pub unregistered_hotkeys: Mutex<Vec<String>>,
}

/// Schritt-Daten für das Frontend (ohne Bild-Bytes; Bilder werden einzeln
/// als `data:`-URI geladen).
#[derive(Debug, Clone, Serialize)]
pub struct StepDto {
    index: usize,
    id: String,
    action: String,
    text: String,
    has_override: bool,
    has_image: bool,
    /// Greenshot-lite Markierungen (normalisierte 0–1-Koordinaten).
    overlays: Vec<Overlay>,
    /// Sichtbarer Ausschnitt, `null` ist das ganze Bild.
    crop: Option<Crop>,
}

fn action_label(action: &Action) -> String {
    match action {
        Action::Click => "click".to_owned(),
        Action::DoubleClick => "double_click".to_owned(),
        Action::RightClick => "right_click".to_owned(),
        Action::KeyCombo { combo } => format!("key_combo {combo}"),
        Action::Scroll { direction, amount } => format!("scroll {direction:?} ×{amount}"),
        Action::TextInput => "text_input".to_owned(),
        Action::ManualNote => "manual_note".to_owned(),
    }
}

fn to_dtos(file: &StepsFile) -> Vec<StepDto> {
    file.guide
        .steps
        .iter()
        .enumerate()
        .map(|(index, step)| StepDto {
            index,
            id: step.id.clone(),
            action: action_label(&step.action),
            text: step.text.effective().to_owned(),
            has_override: step.text.override_text.is_some(),
            has_image: file.images.contains_key(&step.id),
            overlays: step.overlays.clone(),
            crop: step.crop,
        })
        .collect()
}

// --- Aufnahme-Steuerung -------------------------------------------------------

/// Steuert die Aufnahme: `start` | `pause` | `resume` | `stop` | `discard`.
/// `stop` meldet das Ergebnis über `recorder://finished`, ein gescheiterter
/// `start` einen [`CommandError`] über `recorder://start-failed`.
#[tauri::command]
pub fn recorder_command(state: State<'_, AppState>, name: String) -> Result<(), CommandError> {
    let command = match name.as_str() {
        "start" => Command::Start,
        "pause" => Command::Pause,
        "resume" => Command::Resume,
        "stop" => Command::Stop,
        "discard" => Command::Discard,
        other => {
            return Err(CommandError::new(
                ErrorCode::Internal,
                format_args!("unknown recorder command {other}"),
            ))
        }
    };
    state.recorder.send(command).ok();
    Ok(())
}

/// Holt das Hauptfenster aus dem Tray nach vorn.
#[tauri::command]
pub fn show_main_window(app: AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        window.unminimize().ok();
        window.show().ok();
        window.set_focus().ok();
    }
}

/// Beendet die App. Das Tray und Alt+F4 fragen über `app://quit-requested`
/// zuerst das Frontend, damit eine Aufnahme oder eine bearbeitete Anleitung
/// nicht still verloren geht. Die Hooks sind vorher gelöst.
#[tauri::command]
pub fn quit_app(app: AppHandle) {
    if let Some(state) = app.try_state::<AppState>() {
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        if state.recorder.send(Command::Release(done_tx)).is_ok() {
            let _ = done_rx.recv_timeout(std::time::Duration::from_secs(2));
        }
    }
    app.exit(0);
}

/// Nimmt den Pfad eines zweiten Starts genau einmal entgegen.
#[tauri::command]
pub fn take_pending_open(state: State<'_, AppState>) -> Option<String> {
    state
        .pending_open
        .lock()
        .ok()
        .and_then(|mut pending| pending.take())
}

// --- Editor -------------------------------------------------------------------

/// Titel und Schritte der aktuellen Aufnahme.
#[derive(Debug, Clone, Serialize)]
pub struct GuideDto {
    title: String,
    steps: Vec<StepDto>,
}

/// Liefert die aktuelle Aufnahme für den Editor (falls vorhanden).
#[tauri::command]
pub fn get_guide(state: State<'_, AppState>) -> Option<GuideDto> {
    let guard = state.guide.lock().ok()?;
    guard.as_ref().map(|file| GuideDto {
        title: file.guide.title.clone(),
        steps: to_dtos(file),
    })
}

/// Screenshot eines Schritts als `data:`-URI.
#[tauri::command]
pub fn get_step_image(state: State<'_, AppState>, id: String) -> Option<String> {
    let guard = state.guide.lock().ok()?;
    guard
        .as_ref()
        .and_then(|file| file.images.get(&id))
        .map(|png| steps_export::image_data_uri(png))
}

/// Sperrt die Arbeitskopie; ein vergifteter Lock wird `internal`.
fn lock_guide(state: &AppState) -> Result<MutexGuard<'_, Option<StepsFile>>, CommandError> {
    state.guide.lock().map_err(CommandError::poisoned("guide"))
}

fn with_guide<T>(
    state: &State<'_, AppState>,
    edit: impl FnOnce(&mut StepsFile) -> T,
) -> Result<T, CommandError> {
    let mut guard = lock_guide(state)?;
    let file = guard.as_mut().ok_or_else(CommandError::no_guide)?;
    Ok(edit(file))
}

/// Setzt den Guide-Titel.
#[tauri::command]
pub fn set_guide_title(state: State<'_, AppState>, title: String) -> Result<(), CommandError> {
    with_guide(&state, |file| {
        file.guide.title = title;
    })
}

/// Setzt den Schritt-Text als User-Override; leerer Text entfernt den
/// Override (der generierte Text gilt wieder).
#[tauri::command]
pub fn set_step_text(
    state: State<'_, AppState>,
    id: String,
    text: String,
) -> Result<(), CommandError> {
    with_guide(&state, |file| {
        if let Some(step) = file.guide.steps.iter_mut().find(|step| step.id == id) {
            let trimmed = text.trim();
            step.text.override_text = if trimmed.is_empty() || trimmed == step.text.generated {
                None
            } else {
                Some(text)
            };
        }
    })
}

/// Setzt den sichtbaren Ausschnitt. `None` zeigt das ganze Bild.
/// Die PNG-Bytes bleiben unangetastet.
#[tauri::command]
pub fn set_step_crop(
    state: State<'_, AppState>,
    id: String,
    crop: Option<Crop>,
) -> Result<Vec<StepDto>, CommandError> {
    let crop = match crop {
        None => None,
        Some(crop) => crop
            .normalized()
            .map_err(|()| CommandError::new(ErrorCode::Internal, "crop is not a number"))?,
    };
    with_guide(&state, |file| {
        if let Some(step) = file.guide.steps.iter_mut().find(|step| step.id == id) {
            step.crop = crop;
        }
        to_dtos(file)
    })
}

/// Ersetzt die Markierungen eines Schritts (Persistenz auf
/// Stroke-Ende / Undo / Clear). Original-PNGs bleiben unangetastet.
#[tauri::command]
pub fn set_step_overlays(
    state: State<'_, AppState>,
    id: String,
    overlays: Vec<Overlay>,
) -> Result<Vec<StepDto>, CommandError> {
    with_guide(&state, |file| {
        if let Some(step) = file.guide.steps.iter_mut().find(|step| step.id == id) {
            step.overlays = overlays;
        }
        to_dtos(file)
    })
}

/// Löscht einen Schritt samt Screenshot.
#[tauri::command]
pub fn delete_step(state: State<'_, AppState>, id: String) -> Result<Vec<StepDto>, CommandError> {
    with_guide(&state, |file| {
        file.guide.steps.retain(|step| step.id != id);
        file.images.remove(&id);
        to_dtos(file)
    })
}

/// Verschiebt einen Schritt an eine neue Position (Reorder).
#[tauri::command]
pub fn move_step(
    state: State<'_, AppState>,
    from: usize,
    to: usize,
) -> Result<Vec<StepDto>, CommandError> {
    with_guide(&state, |file| {
        let steps = &mut file.guide.steps;
        if from < steps.len() && to < steps.len() && from != to {
            let step = steps.remove(from);
            steps.insert(to, step);
        }
        to_dtos(file)
    })
}

/// Speichert die aktuelle Aufnahme als `.steps`-Datei im Anleitungsordner
/// und liefert den Pfad zurück.
#[tauri::command]
pub fn save_guide(app: AppHandle, state: State<'_, AppState>) -> Result<String, CommandError> {
    let path = {
        let guard = lock_guide(&state)?;
        let file = guard.as_ref().ok_or_else(CommandError::no_guide)?;
        let path = guides_output_path(&app, &state, &file.guide)?;
        file.save(&path)
            .map_err(CommandError::with(ErrorCode::SaveFailed))?;
        path.display().to_string()
    };
    remember_recent_path(&app, &state, &path);
    app.emit("guide://saved", &path).ok();
    Ok(path)
}

/// Ergebnis eines erfolgreichen Open.
#[derive(Debug, Clone, Serialize)]
pub struct OpenGuideResult {
    path: String,
    guide: GuideDto,
}

/// Recent-Eintrag für die Bibliothek (fehlende Pfade bleiben sichtbar, grau).
/// Titel, Schrittzahl und Datum stammen aus `guide.json`, ohne die Bilder zu lesen.
#[derive(Debug, Clone, Default, Serialize)]
pub struct RecentGuideEntry {
    path: String,
    missing: bool,
    /// Die Datei ist da, enthält aber keine lesbare Anleitung.
    unreadable: bool,
    title: Option<String>,
    step_count: Option<usize>,
    /// Aufnahmezeitpunkt, Millisekunden seit 1970 (UTC).
    created_at_ms: Option<u64>,
}

impl RecentGuideEntry {
    fn read(path: String) -> Self {
        if !Path::new(&path).is_file() {
            return Self {
                path,
                missing: true,
                ..Self::default()
            };
        }
        match steps_store::open_guide(&path) {
            Ok(guide) => Self {
                path,
                title: Some(guide.title).filter(|title| !title.trim().is_empty()),
                step_count: Some(guide.steps.len()),
                created_at_ms: Some(guide.created_at_ms).filter(|&ms| ms > 0),
                ..Self::default()
            },
            Err(_) => Self {
                path,
                unreadable: true,
                ..Self::default()
            },
        }
    }
}

/// Schmalste und breiteste Vorschau, die die Bibliothek anfordern darf, in Pixeln.
const THUMBNAIL_WIDTH: std::ops::RangeInclusive<u32> = 64..=960;

/// Öffnet eine `.steps`-Datei.
///
/// Fehlt `path`, öffnet Hows einen nativen Dateidialog über `rfd`. Abbrechen
/// ergibt `Ok(None)`. Sonst wird der Pfad geladen, aus Zuletzt geöffnet oder
/// von der Kommandozeile. Fehlercodes sind `not_found`, `not_steps`,
/// `corrupt` und `newer_version`. Die Oberfläche übersetzt sie.
///
/// `async`, damit der Dateidialog nicht den Tauri-Main-Thread blockiert.
#[tauri::command]
pub async fn open_guide(
    app: AppHandle,
    state: State<'_, AppState>,
    path: Option<String>,
) -> Result<Option<OpenGuideResult>, CommandError> {
    let locale = settings_locale(&state);
    let resolved = match path {
        Some(raw) => PathBuf::from(raw),
        None => match pick_steps_path(locale).await? {
            Some(picked) => picked,
            None => return Ok(None),
        },
    };
    // Re-resolve after await so we never hold State across a yield.
    let state = app.state::<AppState>();
    let loaded = load_guide_from_path(&app, &state, &resolved)?;
    Ok(Some(loaded))
}

/// Recent-Liste (existierende + fehlende Pfade) mit den Kopfdaten jeder Anleitung.
#[tauri::command]
pub async fn get_recent_guides(app: AppHandle) -> Result<Vec<RecentGuideEntry>, CommandError> {
    let paths = lock_settings(&app.state::<AppState>())?
        .recent_paths
        .clone();
    tauri::async_runtime::spawn_blocking(move || {
        paths.into_iter().map(RecentGuideEntry::read).collect()
    })
    .await
    .map_err(CommandError::with(ErrorCode::Internal))
}

/// Verkleinerter erster Screenshot einer zuletzt genutzten Anleitung als
/// `data:`-URI. Liest nur `guide.json` und dieses eine Bild aus dem Container.
/// `None` für Pfade außerhalb der Recent-Liste und für Anleitungen ohne Bild.
#[tauri::command]
pub async fn get_guide_thumbnail(
    app: AppHandle,
    path: String,
    width: u32,
) -> Result<Option<String>, CommandError> {
    if !lock_settings(&app.state::<AppState>())?
        .recent_paths
        .contains(&path)
    {
        return Ok(None);
    }
    let width = width.clamp(*THUMBNAIL_WIDTH.start(), *THUMBNAIL_WIDTH.end());
    tauri::async_runtime::spawn_blocking(move || {
        let preview = Preview::open(&path).ok()?;
        steps_export::thumbnail_data_uri(&preview.first_image?, width)
    })
    .await
    .map_err(CommandError::with(ErrorCode::Internal))
}

/// Lädt `path` in den Guide-Slot, aktualisiert Recent, emittiert Review-State.
pub(crate) fn load_guide_from_path(
    app: &AppHandle,
    state: &AppState,
    path: &Path,
) -> Result<OpenGuideResult, CommandError> {
    if let Err(kind) = classify_open_path(path) {
        if kind == OpenErrorKind::NotFound {
            forget_recent_path(app, state, &path.display().to_string());
        }
        return Err(CommandError::new(kind.into(), path.display()));
    }

    let file = StepsFile::open(path)?;
    let path_string = path.display().to_string();
    let dto = GuideDto {
        title: file.guide.title.clone(),
        steps: to_dtos(&file),
    };
    let step_count = dto.steps.len();

    {
        let mut guard = lock_guide(state)?;
        *guard = Some(file);
    }

    remember_recent_path(app, state, &path_string);
    emit_reviewing(app, step_count);

    Ok(OpenGuideResult {
        path: path_string,
        guide: dto,
    })
}

/// Sprache der gespeicherten Einstellungen (Englisch, falls der Lock
/// vergiftet ist).
fn settings_locale(state: &AppState) -> Locale {
    state
        .settings
        .lock()
        .map(|settings| settings.locale())
        .unwrap_or_default()
}

async fn pick_steps_path(locale: Locale) -> Result<Option<PathBuf>, CommandError> {
    tauri::async_runtime::spawn_blocking(move || {
        rfd::FileDialog::new()
            .set_title(locale.text(Key::DialogOpenGuide))
            .add_filter(locale.text(Key::DialogFilterSteps), &["steps"])
            .add_filter(locale.text(Key::DialogFilterAll), &["*"])
            .pick_file()
    })
    .await
    .map_err(CommandError::with(ErrorCode::DialogFailed))
}

fn remember_recent_path(app: &AppHandle, state: &AppState, path: &str) {
    let Ok(mut settings) = state.settings.lock() else {
        return;
    };
    let limit = settings.recent_limit;
    push_recent(&mut settings.recent_paths, path, limit);
    if let Ok(config_dir) = app.path().app_config_dir() {
        let _ = settings::save(&config_dir, &settings);
    }
}

fn forget_recent_path(app: &AppHandle, state: &AppState, path: &str) {
    let Ok(mut settings) = state.settings.lock() else {
        return;
    };
    prune_recent(&mut settings.recent_paths, path);
    if let Ok(config_dir) = app.path().app_config_dir() {
        let _ = settings::save(&config_dir, &settings);
    }
}

fn emit_reviewing(app: &AppHandle, steps: usize) {
    crate::recorder::emit_state_named(app, "reviewing", steps);
    app.emit("guide://opened", ()).ok();
}

/// Exportiert die aktuelle Aufnahme als selbst-enthaltene HTML-Datei
/// (SOP-Modus, Stil aus den Einstellungen) in den Export-Standardordner und
/// liefert den Pfad zurück.
#[tauri::command]
pub fn export_html(app: AppHandle, state: State<'_, AppState>) -> Result<String, CommandError> {
    let style = lock_settings(&state)?.export_style();
    let guard = lock_guide(&state)?;
    let file = guard.as_ref().ok_or_else(CommandError::no_guide)?;
    let path = export_output_path(&app, &state, &file.guide, "html")?;
    steps_export::write_html_export(file, steps_export::ExportMode::Sop, &style, &path)
        .map_err(CommandError::with(ErrorCode::ExportFailed))?;
    Ok(path.display().to_string())
}

/// Exportiert die aktuelle Aufnahme als PDF (SOP-Modus, Format, Rand und Stil
/// aus den Einstellungen) in den Export-Standardordner und liefert den Pfad zurück.
#[tauri::command]
pub fn export_pdf(app: AppHandle, state: State<'_, AppState>) -> Result<String, CommandError> {
    let (page, style) = {
        let settings = lock_settings(&state)?;
        (settings.pdf_page(), settings.export_style())
    };
    let guard = lock_guide(&state)?;
    let file = guard.as_ref().ok_or_else(CommandError::no_guide)?;
    let path = export_output_path(&app, &state, &file.guide, "pdf")?;
    steps_export::write_pdf_export(file, steps_export::ExportMode::Sop, page, &style, &path)
        .map_err(CommandError::with(ErrorCode::ExportFailed))?;
    Ok(path.display().to_string())
}

/// Exportiert Markdown (`guide.md` + `images/`) in den Export-Standardordner.
#[tauri::command]
pub fn export_markdown(app: AppHandle, state: State<'_, AppState>) -> Result<String, CommandError> {
    let style = lock_settings(&state)?.export_style();
    let guard = lock_guide(&state)?;
    let file = guard.as_ref().ok_or_else(CommandError::no_guide)?;
    let directory = export_output_dir(&app, &state, &file.guide, "markdown")?;
    steps_export::write_markdown_export(file, steps_export::ExportMode::Sop, &style, &directory)
        .map_err(CommandError::with(ErrorCode::ExportFailed))?;
    Ok(directory.join("guide.md").display().to_string())
}

/// Exportiert JSON (SOP-Modus) in den Export-Standardordner.
#[tauri::command]
pub fn export_json(app: AppHandle, state: State<'_, AppState>) -> Result<String, CommandError> {
    let guard = lock_guide(&state)?;
    let file = guard.as_ref().ok_or_else(CommandError::no_guide)?;
    let path = export_output_path(&app, &state, &file.guide, "json")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(CommandError::with(ErrorCode::FolderUnavailable))?;
    }
    let body = steps_export::json(file, steps_export::ExportMode::Sop);
    std::fs::write(&path, body).map_err(CommandError::with(ErrorCode::ExportFailed))?;
    Ok(path.display().to_string())
}

/// Neues `.steps`-Ziel im Anleitungsordner (#8), nie eine vorhandene Datei.
fn guides_output_path(
    app: &AppHandle,
    state: &AppState,
    guide: &Guide,
) -> Result<PathBuf, CommandError> {
    let directory = guides_base_dir(app, state)?;
    Ok(unused_file_path(
        &directory.join(format!("{}.steps", file_stem_for(state, guide))),
    ))
}

/// Neuer Export-Zielpfad im Standardordner (#6), nie eine vorhandene Datei.
fn export_output_path(
    app: &AppHandle,
    state: &AppState,
    guide: &Guide,
    extension: &str,
) -> Result<PathBuf, CommandError> {
    let directory = export_base_dir(app, state)?;
    Ok(unused_file_path(&directory.join(format!(
        "{}.{}",
        file_stem_for(state, guide),
        extension
    ))))
}

/// Neues Export-Zielverzeichnis (z. B. Markdown mit `images/`), damit keine
/// Bilder eines früheren Exports liegen bleiben.
fn export_output_dir(
    app: &AppHandle,
    state: &AppState,
    guide: &Guide,
    suffix: &str,
) -> Result<PathBuf, CommandError> {
    let directory = export_base_dir(app, state)?;
    let path =
        unused_dir_path(&directory.join(format!("{}-{}", file_stem_for(state, guide), suffix)));
    std::fs::create_dir_all(&path).map_err(CommandError::with(ErrorCode::FolderUnavailable))?;
    Ok(path)
}

fn documents_base_dir(app: &AppHandle) -> Result<PathBuf, CommandError> {
    let directory = app
        .path()
        .document_dir()
        .or_else(|_| app.path().app_data_dir())
        .map_err(CommandError::with(ErrorCode::FolderUnavailable))?;
    std::fs::create_dir_all(&directory)
        .map_err(CommandError::with(ErrorCode::FolderUnavailable))?;
    Ok(directory)
}

fn export_base_dir(app: &AppHandle, state: &AppState) -> Result<PathBuf, CommandError> {
    let fallback = documents_base_dir(app)?;
    let configured = state
        .settings
        .lock()
        .map(|s| s.default_export_folder.clone())
        .unwrap_or_default();
    let (path, _) = settings::resolve_folder(&configured, fallback);
    std::fs::create_dir_all(&path).map_err(CommandError::with(ErrorCode::FolderUnavailable))?;
    Ok(path)
}

fn guides_base_dir(app: &AppHandle, state: &AppState) -> Result<PathBuf, CommandError> {
    let fallback = documents_base_dir(app)?.join("Steps");
    let configured = state
        .settings
        .lock()
        .map(|s| s.guides_folder.clone())
        .unwrap_or_default();
    let (path, _) = settings::resolve_folder(&configured, fallback);
    std::fs::create_dir_all(&path).map_err(CommandError::with(ErrorCode::FolderUnavailable))?;
    Ok(path)
}

/// Dateiname ohne Endung aus der Dateinamenvorlage der Einstellungen.
fn file_stem_for(state: &AppState, guide: &Guide) -> String {
    let template = state
        .settings
        .lock()
        .map(|settings| settings.file_name_template.clone())
        .unwrap_or_else(|_| Settings::default().file_name_template);
    file_stem(&template, guide, &Stamp::local(guide.created_at_ms))
}

/// Füllt `{title}`, `{app}`, `{date}` und `{time}` ein; Datum und Uhrzeit
/// kommen aus `stamp`. Bleibt nichts Brauchbares übrig, gilt der Name für
/// Anleitungen ohne Titel in der Sprache der Anleitung.
fn file_stem(template: &str, guide: &Guide, stamp: &Stamp) -> String {
    let app = steps_session::app_name(&guide.steps).unwrap_or_default();
    let filled = steps_i18n::fill(
        template,
        &[
            ("title", guide.title.trim()),
            ("app", &app),
            ("date", &stamp.date),
            ("time", &stamp.time),
        ],
    );
    sanitize_file_name(&filled)
        .or_else(|| sanitize_file_name(Locale::resolve(&guide.language).text(Key::ExportUntitled)))
        .unwrap_or_else(|| stamp.date.clone())
}

/// Macht einen Text als Dateinamen verwendbar; `None`, wenn nichts übrig bleibt.
fn sanitize_file_name(text: &str) -> Option<String> {
    let cleaned: String = text
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => ' ',
            other if other.is_control() => ' ',
            other => other,
        })
        .collect();
    let collapsed: String = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    let trimmed = collapsed.trim().trim_matches('.').trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(dodge_reserved_windows_name(trimmed))
}

/// `CON`, `NUL.txt` und `lpt3.steps` sind unter Windows Geräte, keine Dateien.
/// Der reservierte Namensanfang bekommt ein `_`.
fn dodge_reserved_windows_name(name: &str) -> String {
    let (base, rest) = name.split_once('.').unwrap_or((name, ""));
    if !is_reserved_windows_basename(base) {
        return name.to_owned();
    }
    if rest.is_empty() {
        format!("{base}_")
    } else {
        format!("{base}_.{rest}")
    }
}

fn is_reserved_windows_basename(name: &str) -> bool {
    let upper = name.to_ascii_uppercase();
    match upper.as_bytes() {
        b"CON" | b"PRN" | b"AUX" | b"NUL" => true,
        [b'C', b'O', b'M', digit] | [b'L', b'P', b'T', digit] => matches!(digit, b'1'..=b'9'),
        _ => false,
    }
}

// --- Einstellungen --------------------------------------------------------------

/// DTO mit aufgelösten Pfaden + Fallback-Status.
#[derive(Debug, Clone, Serialize)]
pub struct SettingsView {
    #[serde(flatten)]
    settings: Settings,
    /// Anzeigepfad Export-Standardordner (ggf. Fallback).
    export_folder_display: String,
    export_folder_fallback: bool,
    /// Export-Ordner liegt unter OneDrive (Export-Sheet warnt).
    export_folder_synced: bool,
    /// Anzeigepfad Anleitungsordner (ggf. Fallback).
    guides_folder_display: String,
    guides_folder_fallback: bool,
    /// Anleitungsordner liegt unter OneDrive.
    guides_folder_synced: bool,
    /// Hotkey-Slots, die beim Start nicht registriert werden konnten.
    unregistered_hotkeys: Vec<String>,
    /// Sprache, die `language` gerade ergibt; `system` ist aufgelöst.
    resolved_language: &'static str,
    /// Titelvorlage der eingestellten Sprache; gilt, solange
    /// `title_template` leer ist.
    language_title_template: &'static str,
    /// Datum und Uhrzeit von jetzt, so wie `{date}` und `{time}` sie für
    /// die Vorschau einsetzen.
    preview_date: String,
    preview_time: String,
}

/// Standardwerte und Grenzen für die UI, damit sie keine eigenen Werte kennt.
#[derive(Debug, Clone, Serialize)]
pub struct SettingsDefaults {
    defaults: Settings,
    limits: settings::Limits,
    pdf_papers: Vec<&'static str>,
}

enum FolderKind {
    Export,
    Guides,
}

pub(crate) enum HotkeySlot {
    Toggle,
    Pause,
}

impl HotkeySlot {
    /// Slot-Name wie im Frontend (`record` | `pause`).
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Self::Toggle => "record",
            Self::Pause => "pause",
        }
    }
}

fn lock_settings(state: &AppState) -> Result<MutexGuard<'_, Settings>, CommandError> {
    state
        .settings
        .lock()
        .map_err(CommandError::poisoned("settings"))
}

/// Speichert die Einstellungen und reicht sie an Recorder und Tray weiter.
fn commit_settings(app: &AppHandle, settings: &Settings) -> Result<(), CommandError> {
    if let Ok(config_dir) = app.path().app_config_dir() {
        settings::save(&config_dir, settings)
            .map_err(CommandError::with(ErrorCode::SettingsNotSaved))?;
    }
    let state = app.state::<AppState>();
    state
        .recorder
        .send(Command::Configure(RecorderConfig::from_settings(settings)))
        .ok();
    if let Some(tray) = app.try_state::<TrayMenu>() {
        tray.apply(settings.locale());
    }
    Ok(())
}

fn update_settings(
    app: &AppHandle,
    state: &AppState,
    update: impl FnOnce(&mut Settings),
) -> Result<(), CommandError> {
    let mut settings = lock_settings(state)?;
    update(&mut settings);
    commit_settings(app, &settings)
}

fn settings_view_for(
    app: &AppHandle,
    state: &AppState,
    settings: &Settings,
) -> Result<SettingsView, CommandError> {
    let docs = documents_base_dir(app)?;
    let (export_path, export_fallback) =
        settings::resolve_folder(&settings.default_export_folder, docs.clone());
    let (guides_path, guides_fallback) =
        settings::resolve_folder(&settings.guides_folder, docs.join("Steps"));
    let unregistered_hotkeys = state
        .unregistered_hotkeys
        .lock()
        .map(|slots| slots.clone())
        .unwrap_or_default();
    let locale = settings.locale();
    let now = Stamp::local(steps_capture::now_ms());
    Ok(SettingsView {
        settings: settings.clone(),
        export_folder_display: export_path.display().to_string(),
        export_folder_fallback: export_fallback,
        export_folder_synced: settings::is_cloud_synced(&export_path),
        guides_folder_display: guides_path.display().to_string(),
        guides_folder_fallback: guides_fallback,
        guides_folder_synced: settings::is_cloud_synced(&guides_path),
        unregistered_hotkeys,
        resolved_language: locale.code(),
        language_title_template: locale.text(Key::GuideTitle),
        preview_date: now.date,
        preview_time: now.time,
    })
}

/// Liefert die aktuellen Einstellungen inkl. aufgelöster Ordnerpfade.
#[tauri::command]
pub fn get_settings(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SettingsView, CommandError> {
    let settings = lock_settings(&state)?.clone();
    settings_view_for(&app, &state, &settings)
}

/// Standardwerte, Grenzen und Auswahllisten der Einstellungen.
#[tauri::command]
pub fn get_settings_defaults() -> SettingsDefaults {
    SettingsDefaults {
        defaults: Settings::default(),
        limits: settings::LIMITS,
        pdf_papers: steps_export::Paper::ALL
            .iter()
            .map(|paper| paper.code())
            .collect(),
    }
}

/// Ändert einen Wert; Zahlen werden auf die Grenzen begrenzt.
#[tauri::command]
pub fn change_setting(
    app: AppHandle,
    state: State<'_, AppState>,
    change: settings::SettingChange,
) -> Result<SettingsView, CommandError> {
    let mut settings = lock_settings(&state)?;
    settings.apply(change);
    commit_settings(&app, &settings)?;
    settings_view_for(&app, &state, &settings)
}

/// Setzt alle Werte unter „Erweitert“ auf ihre Standards zurück.
#[tauri::command]
pub fn reset_advanced_settings(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<SettingsView, CommandError> {
    let mut settings = lock_settings(&state)?;
    settings.reset_advanced();
    commit_settings(&app, &settings)?;
    settings_view_for(&app, &state, &settings)
}

fn chords_conflict(a: &str, b: &str) -> bool {
    a.trim().eq_ignore_ascii_case(b.trim())
}

fn set_control_hotkey(
    app: &AppHandle,
    state: &AppState,
    slot: HotkeySlot,
    hotkey: String,
) -> Result<(), CommandError> {
    let mut settings = lock_settings(state)?;
    let (previous, other) = match slot {
        HotkeySlot::Toggle => (settings.hotkey.clone(), settings.pause_hotkey.clone()),
        HotkeySlot::Pause => (settings.pause_hotkey.clone(), settings.hotkey.clone()),
    };
    if !crate::settings::is_recording_shortcut(&hotkey) {
        return Err(CommandError::new(ErrorCode::HotkeyInvalid, &hotkey));
    }
    if chords_conflict(&hotkey, &other) {
        return Err(CommandError::new(ErrorCode::HotkeyConflict, &hotkey));
    }
    // Erst den alten Hotkey freigeben, dann den neuen registrieren.
    // Umgekehrt schlägt die Registrierung fehl, wenn sich nur die Taste
    // ändert und die Modifikatoren gleich bleiben.
    if previous != hotkey {
        crate::unregister_hotkey(app, &previous);
    }
    if let Err(error) = crate::register_hotkey(app, &hotkey) {
        // Vorherige Kombination wiederherstellen, damit sie nicht verloren geht.
        let _ = crate::register_hotkey(app, &previous);
        return Err(error);
    }
    match slot {
        HotkeySlot::Toggle => settings.hotkey = hotkey,
        HotkeySlot::Pause => settings.pause_hotkey = hotkey,
    }
    if let Ok(mut unregistered) = state.unregistered_hotkeys.lock() {
        unregistered.retain(|name| name != slot.name());
    }
    commit_settings(app, &settings)
}

/// Ändert den globalen Aufnahme-Hotkey (validiert, registriert um,
/// persistiert).
#[tauri::command]
pub fn set_hotkey(
    app: AppHandle,
    state: State<'_, AppState>,
    hotkey: String,
) -> Result<(), CommandError> {
    set_control_hotkey(&app, &state, HotkeySlot::Toggle, hotkey)
}

/// Ändert den globalen Pause-Hotkey (validiert, registriert um,
/// persistiert).
#[tauri::command]
pub fn set_pause_hotkey(
    app: AppHandle,
    state: State<'_, AppState>,
    hotkey: String,
) -> Result<(), CommandError> {
    set_control_hotkey(&app, &state, HotkeySlot::Pause, hotkey)
}

/// UI-Sprache setzen (`system` oder ein Code aus dem Sprach-Crate,
/// unbekannte fallen auf Englisch) und persistieren. Die Antwort trägt die Titelvorlage der neuen
/// Sprache mit.
#[tauri::command]
pub fn set_language(
    app: AppHandle,
    state: State<'_, AppState>,
    language: String,
) -> Result<SettingsView, CommandError> {
    update_settings(&app, &state, |settings| {
        settings.language = settings::normalize_language(&language);
    })?;
    let settings = lock_settings(&state)?.clone();
    settings_view_for(&app, &state, &settings)
}

/// Standard-Exportformat (`html` | `pdf` | `steps`).
#[tauri::command]
pub fn set_export_format(
    app: AppHandle,
    state: State<'_, AppState>,
    format: String,
) -> Result<(), CommandError> {
    update_settings(&app, &state, |settings| {
        settings.export_format = settings::normalize_export_format(&format);
    })
}

/// Nach Export öffnen (bool).
#[tauri::command]
pub fn set_open_after_export(
    app: AppHandle,
    state: State<'_, AppState>,
    open: bool,
) -> Result<(), CommandError> {
    update_settings(&app, &state, |settings| {
        settings.open_after_export = open;
    })
}

/// Theme (`system` | `light` | `dark`).
#[tauri::command]
pub fn set_theme(
    app: AppHandle,
    state: State<'_, AppState>,
    theme: String,
) -> Result<(), CommandError> {
    update_settings(&app, &state, |settings| {
        settings.theme = settings::normalize_theme(&theme);
    })
}

fn set_folder_field(
    app: &AppHandle,
    state: &AppState,
    path: String,
    which: FolderKind,
) -> Result<SettingsView, CommandError> {
    let trimmed = path.trim().to_owned();
    let docs = documents_base_dir(app)?;
    let fallback = match which {
        FolderKind::Export => docs,
        FolderKind::Guides => docs.join("Steps"),
    };

    let mut settings = lock_settings(state)?;

    if trimmed.is_empty() {
        match which {
            FolderKind::Export => settings.default_export_folder.clear(),
            FolderKind::Guides => settings.guides_folder.clear(),
        }
        commit_settings(app, &settings)?;
        return settings_view_for(app, state, &settings);
    }

    if !settings::folder_is_usable(Path::new(&trimmed)) {
        // Vorherigen gültigen Wert behalten; Frontend zeigt Fallback-Pfad.
        return Err(
            CommandError::new(ErrorCode::InvalidFolder, &trimmed).at_path(fallback.display())
        );
    }

    match which {
        FolderKind::Export => settings.default_export_folder = trimmed,
        FolderKind::Guides => settings.guides_folder = trimmed,
    }
    commit_settings(app, &settings)?;
    settings_view_for(app, state, &settings)
}

/// Standard-Exportordner (#6). Bei ungültig: Err `invalid_folder` mit dem Ersatzordner in `path`.
#[tauri::command]
pub fn set_default_export_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<SettingsView, CommandError> {
    set_folder_field(&app, &state, path, FolderKind::Export)
}

/// Anleitungsordner (#8). Bei ungültig: Err `invalid_folder` mit dem Ersatzordner in `path`.
#[tauri::command]
pub fn set_guides_folder(
    app: AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> Result<SettingsView, CommandError> {
    set_folder_field(&app, &state, path, FolderKind::Guides)
}

/// Nativer Ordner-Dialog. Abbrechen ergibt `Ok(None)`.
#[tauri::command]
pub async fn pick_folder(
    state: State<'_, AppState>,
    title: Option<String>,
) -> Result<Option<String>, CommandError> {
    let dialog_title = title.unwrap_or_else(|| {
        settings_locale(&state)
            .text(Key::DialogChooseFolder)
            .to_owned()
    });
    tauri::async_runtime::spawn_blocking(move || {
        rfd::FileDialog::new()
            .set_title(dialog_title)
            .pick_folder()
            .map(|path| path.display().to_string())
    })
    .await
    .map_err(CommandError::with(ErrorCode::DialogFailed))
}

/// Öffnet einen lokalen Pfad im OS-Dateimanager / Standard-App (Nach-Export).
#[tauri::command]
pub fn reveal_path(path: String) -> Result<(), CommandError> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err(CommandError::new(ErrorCode::PathMissing, path.display()));
    }
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(if path.is_dir() {
                path.as_os_str().to_os_string()
            } else {
                // Select file in Explorer when possible.
                let mut arg = std::ffi::OsString::from("/select,");
                arg.push(path.as_os_str());
                arg
            })
            .spawn()
            .map_err(CommandError::with(ErrorCode::RevealFailed))?;
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(CommandError::with(ErrorCode::RevealFailed))?;
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(CommandError::with(ErrorCode::RevealFailed))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{file_stem, sanitize_file_name, RecentGuideEntry};
    use steps_session::Stamp;
    use steps_store::{Action, ElementInfo, Guide, Step, StepText, StepsFile};

    #[test]
    fn dateinamen_werden_bereinigt() {
        assert_eq!(
            sanitize_file_name("Netzlaufwerk verbinden").as_deref(),
            Some("Netzlaufwerk verbinden")
        );
        assert_eq!(
            sanitize_file_name("a/b\\c:d*e?").as_deref(),
            Some("a b c d e")
        );
        assert_eq!(sanitize_file_name("  .. "), None);
        assert_eq!(sanitize_file_name(""), None);
        assert_eq!(
            sanitize_file_name("Pull requests <> \"x\"").as_deref(),
            Some("Pull requests x")
        );
        assert_eq!(sanitize_file_name("CON").as_deref(), Some("CON_"));
        assert_eq!(sanitize_file_name("nul.pdf").as_deref(), Some("nul_.pdf"));
        assert_eq!(
            sanitize_file_name("CON.steps").as_deref(),
            Some("CON_.steps")
        );
        assert_eq!(sanitize_file_name("com1").as_deref(), Some("com1_"));
        assert_eq!(sanitize_file_name("LPT9.txt").as_deref(), Some("LPT9_.txt"));
        assert_eq!(sanitize_file_name("AUX").as_deref(), Some("AUX_"));
        assert_eq!(sanitize_file_name("PRN").as_deref(), Some("PRN_"));
        assert_eq!(sanitize_file_name("Report").as_deref(), Some("Report"));
        assert_eq!(sanitize_file_name("COM0").as_deref(), Some("COM0"));
        assert_eq!(sanitize_file_name("CONSOLE").as_deref(), Some("CONSOLE"));
    }

    /// 2026-09-17 00:00 UTC
    const DAY_MS: u64 = 1_789_603_200_000;

    fn utc() -> Stamp {
        Stamp::with_offset(DAY_MS, 0)
    }

    #[test]
    fn dateiname_folgt_der_vorlage() {
        let guide = Guide::new("Notepad: Save", DAY_MS, "en");
        assert_eq!(file_stem("{title}", &guide, &utc()), "Notepad Save");
        assert_eq!(
            file_stem("{date} {title}", &guide, &utc()),
            "2026-09-17 Notepad Save"
        );
        assert_eq!(file_stem("{nope}", &guide, &utc()), "{nope}");
    }

    #[test]
    fn dateiname_nutzt_ortszeit_app_und_uhrzeit() {
        let mut guide = Guide::new("Save", DAY_MS, "en");
        guide.steps.push(Step {
            id: "step-0001".to_owned(),
            action: Action::Click,
            timestamp_ms: DAY_MS,
            position: None,
            monitor: None,
            element: Some(ElementInfo {
                app_name: Some("Notepad".to_owned()),
                ..ElementInfo::default()
            }),
            text: StepText::generated("Click"),
            overlays: Vec::new(),
            crop: None,
        });
        let template = "{date} {time} {app} {title}";
        assert_eq!(
            file_stem(template, &guide, &utc()),
            "2026-09-17 00.00 Notepad Save"
        );
        // 00:00 UTC ist in New York (UTC-4) noch der Vortag.
        assert_eq!(
            file_stem(template, &guide, &Stamp::with_offset(DAY_MS, -4 * 3600)),
            "2026-09-16 20.00 Notepad Save"
        );
    }

    #[test]
    fn ohne_titel_gilt_der_name_der_sprache() {
        assert_eq!(
            file_stem("{title}", &Guide::new("", DAY_MS, "de"), &utc()),
            "Unbenannte Anleitung"
        );
        assert_eq!(
            file_stem("{title}", &Guide::new(" ", DAY_MS, "en"), &utc()),
            "Untitled guide"
        );
        assert_eq!(
            file_stem("{title} {date}", &Guide::new("", DAY_MS, "fr"), &utc()),
            "2026-09-17"
        );
    }

    #[test]
    fn recent_eintraege_lesen_kopfdaten_und_melden_fehler() {
        let dir = std::env::temp_dir().join(format!("hows-recent-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("Temp-Verzeichnis");
        let path_of = |name: &str| dir.join(name).display().to_string();

        let mut guide = Guide::new("Drucker einrichten", DAY_MS, "de");
        guide.steps = Vec::new();
        StepsFile::new(guide)
            .save(path_of("ok.steps"))
            .expect("Speichern");
        std::fs::write(path_of("kaputt.steps"), b"kein zip").expect("Schreiben");

        let ok = RecentGuideEntry::read(path_of("ok.steps"));
        assert!(!ok.missing && !ok.unreadable);
        assert_eq!(ok.title.as_deref(), Some("Drucker einrichten"));
        assert_eq!((ok.step_count, ok.created_at_ms), (Some(0), Some(DAY_MS)));

        let broken = RecentGuideEntry::read(path_of("kaputt.steps"));
        assert!(broken.unreadable && !broken.missing && broken.title.is_none());

        let gone = RecentGuideEntry::read(path_of("fehlt.steps"));
        assert!(gone.missing && !gone.unreadable && gone.step_count.is_none());

        std::fs::remove_dir_all(&dir).ok();
    }
}
