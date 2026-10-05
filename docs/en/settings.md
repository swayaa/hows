English | [Deutsch](../de/settings.md)

# Settings

Click **Settings** in the library. Settings are saved on your PC, and changes apply right away. Once you change a value that has a default, such as the paper size, the style, or a value under **Advanced**, a <!-- fact:ui.resetValue -->**Reset**<!-- /fact --> button next to it restores the default.

## Recording

| Setting | Default | What it does |
|---|---|---|
| <!-- fact:ui.hotkeyRecord -->Start or stop recording<!-- /fact --> | <!-- fact:settings.hotkey keys -->`Ctrl+Shift+R`<!-- /fact --> | Global shortcut that starts a recording, or stops it and opens the editor. |
| <!-- fact:ui.hotkeyPause -->Pause or resume<!-- /fact --> | <!-- fact:settings.pause_hotkey keys -->`Ctrl+Shift+P`<!-- /fact --> | Global shortcut that pauses or resumes a running recording. |

To change a shortcut, click its field and press the new combination. See [Keyboard shortcuts](hotkeys.md#change-a-shortcut) for the rules.

## Export

| Setting | Default | What it does |
|---|---|---|
| <!-- fact:ui.exportDefault -->Default format<!-- /fact --> | <!-- fact:settings.export_format ui:exportFormat -->HTML<!-- /fact --> | Format the export sheet starts with: <!-- fact:app.export_formats ui:exportFormat -->HTML, PDF, or `.steps`<!-- /fact -->. |
| <!-- fact:ui.openAfterExport -->Open after export<!-- /fact --> | <!-- fact:settings.open_after_export bool -->On<!-- /fact --> | Shows the exported file in File Explorer. |
| <!-- fact:ui.exportCredit -->Show “Created with Hows” in exports<!-- /fact --> | <!-- fact:settings.export_credit bool -->On<!-- /fact --> | Adds a short credit line to HTML and PDF exports. See [Export](export.md#look-and-credit-line). |
| <!-- fact:ui.pdfPaper -->PDF paper size<!-- /fact --> | <!-- fact:settings.pdf_paper ui:paperNames -->A4<!-- /fact --> | <!-- fact:export.papers ui:paperNames -->A4 or US Letter<!-- /fact -->. |
| <!-- fact:ui.defaultFolder -->Export folder<!-- /fact --> | Documents | Where HTML and PDF exports go. **Browse…** picks another folder. |

## Appearance

| Setting | Default | What it does |
|---|---|---|
| <!-- fact:ui.language -->Language<!-- /fact --> | <!-- fact:settings.language ui:language -->Match Windows<!-- /fact --> | Language of the interface, the notification area menu, file dialogs, and new step texts. <!-- fact:ui.languageSystem -->**Match Windows**<!-- /fact --> uses the Windows display language, or English if Hows doesn't offer it. Available: <!-- fact:i18n.languages -->English, Deutsch, Français, Español, Italiano, Português (Brasil), Nederlands<!-- /fact -->. |
| <!-- fact:ui.theme -->Theme<!-- /fact --> | <!-- fact:settings.theme ui:theme -->Match Windows<!-- /fact --> | Light, dark, or the same as Windows. |
| <!-- fact:ui.brand -->Style<!-- /fact --> | <!-- fact:settings.brand ui:brandNames -->Sage<!-- /fact --> | Color preset of the app: <!-- fact:brands.codes ui:brandNames -->Sage, Ink, or Ember<!-- /fact -->, each with a light and a dark palette. |
| <!-- fact:ui.accentColor -->Accent color<!-- /fact --> | The accent of the style | Your own accent color. Hows darkens or lightens it until text on it stays readable. |

A language change applies at once to the interface, the notification area menu, and file dialogs. Step texts of guides you already recorded stay as they are; the new language applies from the next recording. With **Match Windows**, Hows follows a new Windows display language after you restart it.

Style and accent color also apply to HTML and PDF exports. See [Export](export.md#look-and-credit-line).

## Folders

| Setting | Default | What it does |
|---|---|---|
| <!-- fact:ui.guidesFolder -->Guides folder<!-- /fact --> | <!-- fact:app.guides_subfolder -->`Steps`<!-- /fact --> in your Documents folder | Where `.steps` files are saved. **Browse…** picks another folder. |

If Hows cannot use a folder, it saves to the default folder instead and shows a note.

## Advanced

Open <!-- fact:ui.groupAdvanced -->**Advanced**<!-- /fact --> for values you rarely need. Each starts with a sensible default; <!-- fact:ui.resetAdvanced -->**Reset all advanced settings**<!-- /fact --> restores all of them.

| Setting | Default | Range | What it does |
|---|---|---|---|
| <!-- fact:ui.typingPause -->Typing pause that starts a new step<!-- /fact --> | <!-- fact:settings.typing_pause_ms seconds -->3<!-- /fact --> s | <!-- fact:limits.typing_pause_ms.min seconds -->0.5<!-- /fact --> to <!-- fact:limits.typing_pause_ms.max seconds -->10<!-- /fact --> s | Typing after a longer pause starts a new typing step. |
| <!-- fact:ui.scrollPause -->Scrolling pause that starts a new step<!-- /fact --> | <!-- fact:settings.scroll_pause_ms seconds -->1.5<!-- /fact --> s | <!-- fact:limits.scroll_pause_ms.min seconds -->0.25<!-- /fact --> to <!-- fact:limits.scroll_pause_ms.max seconds -->5<!-- /fact --> s | Scrolling after a longer pause starts a new scroll step. |
| <!-- fact:ui.recentLimit -->Recent guides in the library<!-- /fact --> | <!-- fact:settings.recent_limit -->8<!-- /fact --> | <!-- fact:limits.recent_limit.min -->1<!-- /fact --> to <!-- fact:limits.recent_limit.max -->30<!-- /fact --> | How many guides the library lists. |
| <!-- fact:ui.titleTemplate -->Title of new recordings<!-- /fact --> | Empty (the default of your language) | | Pattern for the suggested title. `{app}` becomes the app you used, `{date}` the day, and `{time}` the time of the recording. |
| <!-- fact:ui.fileNameTemplate -->File name<!-- /fact --> | <!-- fact:settings.file_name_template -->`{title}`<!-- /fact --> | | Pattern for export file names. `{title}` becomes the guide title, `{app}` the app you used, `{date}` the day, and `{time}` the time of the recording. |
| <!-- fact:ui.pdfMargin -->PDF page margin<!-- /fact --> | <!-- fact:settings.pdf_margin_mm -->18<!-- /fact --> mm | <!-- fact:limits.pdf_margin_mm.min -->5<!-- /fact --> to <!-- fact:limits.pdf_margin_mm.max -->40<!-- /fact --> mm | Margin around each PDF page. |
| <!-- fact:ui.annotationStroke -->Line width of new marks<!-- /fact --> | <!-- fact:settings.annotation_stroke -->2.5<!-- /fact --> px | <!-- fact:limits.annotation_stroke.min -->1<!-- /fact --> to <!-- fact:limits.annotation_stroke.max -->8<!-- /fact --> px | Starting line width of new annotations. |

Both patterns show a live example under the field. Dates and times use the time zone of your PC. `{date}` is written as year, month, and day, for example `2026-09-30`. `{time}` is written as hours and minutes on a 24-hour clock, for example `14.05`; it uses a period because file names can't contain a colon. The file name pattern cannot create folders: `/` and `\` become spaces.

Numbers use the decimal mark of the interface language, for example 1.5 in English and 1,5 in German. You can type either mark. With the field focused, `Up` and `Down` raise or lower the value by one step.

Values outside the range are set to the nearest allowed value. This also applies if you edit the settings file by hand.

## Where settings are stored

Hows keeps its settings in a <!-- fact:app.settings_file -->`settings.json`<!-- /fact --> file in its folder under `%APPDATA%`. The file holds the values on this page and the list of recent guides. See [Privacy](privacy.md#what-stays-on-your-pc).

---

[Documentation overview](README.md) · Next: [Keyboard shortcuts](hotkeys.md)
