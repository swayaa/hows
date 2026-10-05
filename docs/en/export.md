English | [Deutsch](../de/export.md)

# Export and share

Click <!-- fact:ui.exportPrimary -->**Export**<!-- /fact --> in the editor. The <!-- fact:ui.exportTitle -->**Export guide**<!-- /fact --> sheet opens:

1. Pick a format: <!-- fact:ui.exportHtml -->**Web page (HTML)**<!-- /fact -->, <!-- fact:ui.exportPdf -->**PDF document**<!-- /fact -->, or <!-- fact:ui.exportSteps -->**Hows file (.steps)**<!-- /fact -->. The sheet starts with your default format from [Settings](settings.md#export).
2. Check the folder next to <!-- fact:ui.exportTarget -->**Saves to**<!-- /fact -->. <!-- fact:ui.exportChangeFolder -->**Change…**<!-- /fact --> picks another one and remembers it.
3. Leave <!-- fact:ui.exportOpenAfter -->**Open when done**<!-- /fact --> selected if you want File Explorer to show the file afterward.
4. Click **Export**. A message shows where the file was saved.

Press `Esc` or click <!-- fact:ui.closeSheet -->**Close**<!-- /fact --> to leave the sheet without exporting.

## Formats

### Web page (HTML)

One self-contained `.html` file with all screenshots and the Hows font embedded. It opens in any browser, without an internet connection and without Hows. This is the recommended format for sharing by email or chat, or on an intranet. If the reader's Windows uses dark mode, the page shows the dark palette of your style. Printouts always use the light one.

### PDF document

A PDF for printing or archiving. The paper size (<!-- fact:export.papers ui:paperNames -->A4 or US Letter<!-- /fact -->) and the page margin come from [Settings](settings.md#export). The Hows font is embedded, so umlauts, accents, and letters such as the Polish ł print as they appear in the editor. The font has no arrow characters, so an arrow is written as a hyphen and a greater-than sign in the PDF.

### Hows file (.steps)

The editable original: all steps, texts, marks, and the unmarked screenshots. Export this format to keep working on a guide later. It is saved to your guides folder, not the export folder, and appears under <!-- fact:ui.recent -->**Recent guides**<!-- /fact --> in the library. The format is open and documented in [The `.steps` file format](file-format.md).

### Markdown and JSON

The app does not offer these formats yet. The command-line tool `steps-cli` writes them from any `.steps` file:

- **Markdown**: a folder with `guide.md` and an `images` folder, handy for wikis and issue trackers.
- **JSON**: the guide as data, for scripts and other tools.

See [Command line](cli.md#export-a-guide).

## Look and credit line

HTML and PDF exports use the <!-- fact:ui.brand -->**Style**<!-- /fact --> and <!-- fact:ui.accentColor -->**Accent color**<!-- /fact --> from [Settings](settings.md#appearance). If your own accent color is hard to read on the page, Hows darkens or lightens it the same way the app does.

By default, a short line reads “<!-- fact:i18n.export.credit -->Created with Hows<!-- /fact -->” in the language of the guide. It appears at the end of an HTML export and in the footer of every PDF page. It contains no link and no data about you. To leave it out, clear <!-- fact:ui.exportCredit -->**Show “Created with Hows” in exports**<!-- /fact --> under [Settings](settings.md#export). The `.steps` file never contains this line.

`steps-cli` writes the same look and the same line, in Markdown too. It uses the default style unless you pass `--brand`, `--accent`, or `--no-credit`. See [Command line](cli.md#export-a-guide).

## What an export contains

Exports from the app are written as a guide for readers: the title, the numbered step texts, and the screenshots with your marks drawn in. They leave out the separate technical fields, such as window title, click position, timestamp, and system details. A step text can still name a visible button or field and the window or page title; see [Privacy](privacy.md#exports).

`steps-cli` can also export in **bug-report mode**, which adds those technical details for developers. See [Command line](cli.md#guide-mode-and-bug-report-mode) and [Privacy](privacy.md#exports).

## Folders and file names

| Format | Folder | Default |
|---|---|---|
| HTML, PDF | Export folder | Your Documents folder |
| `.steps` | Guides folder | <!-- fact:app.guides_subfolder -->`Steps`<!-- /fact --> in your Documents folder |

The file name follows the file name pattern under [Settings](settings.md#advanced), by default the guide title. **An export never replaces an existing file.** If the name is taken, Hows adds a number, for example `Guide (2).html`, then `Guide (3).html`. The same applies to `.steps` files. Every save creates a new file next to the earlier one. Add `{date}` and `{time}` to the file name pattern if you prefer names without numbers.

If a folder cannot be used, Hows saves to the default folder instead and tells you so.

## OneDrive warning

Windows often moves the Documents folder into OneDrive. If the target folder is synced by OneDrive, the sheet warns you before you export. The screenshots would end up in the cloud. Click **Change…** to pick a local folder instead.

---

[Documentation overview](README.md) · Next: [Settings](settings.md)
