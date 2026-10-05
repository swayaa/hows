English | [Deutsch](CHANGELOG.de.md)

# Changelog

All notable changes to Hows are documented here. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [0.1.0] - 2026-10-06

- Starting or stopping a recording with the shortcut while Hows is in the background, minimized, or in the notification area keeps a single window and shows the recording bar at the top. A second launch no longer opens another Hows.
- A click on Hows, including Stop on the recording bar, is not recorded.
- The library no longer shows “Local only” or “Saved on this PC”.
- The color of new marks is chosen on the annotate bar and remembered. It is no longer under Appearance.
- The guide title, Annotate, and Export sit on one line. The step text and Delete step sit above the screenshot. Enter keeps the step text and leaves the field. Shift+Enter starts a new line. A hint under the field says so.
- Crop hides the edges of a screenshot without deleting them. It sits on the annotate bar. Drag a corner of the frame. Show full image brings the edges back. Exports show the window. The picture in the guide file stays whole.
- A scroll step names the window and how many wheel notches you scrolled, for example *Scroll down 3 in “Documents”*. While you keep scrolling in that window and direction, Hows adds to the same step and does not inspect the control under the pointer again.
- `Alt+F4` quits Hows. Any other close of the recording bar, for example from the taskbar, pauses the recording and hides the bar. Hows keeps running and records nothing while the bar is hidden. If the recording was already paused, it stays paused. Quitting asks first when that would drop steps already recorded or unsaved work in the editor. Opening a `.steps` file while Hows is already running asks before it replaces unsaved work, and does not replace a recording.

### Documentation

- Privacy and README pages state that the person who records is responsible for other people's screens, chats, and for sharing; that the MIT license covers Hows, not the captured interface; and that “no network” is true of the running app. The installers download WebView2 from Microsoft when it is missing.

### Recording (Windows)

- Clicks, double-clicks, right-clicks, and modifier shortcuts become steps with a screenshot and UI Automation metadata.
- Typing becomes one “Type your text” step per field until you pause; the characters are never recorded.
- Scrolling is merged per window and direction until a pause; touchpad movement counts too.
- Double-click detection uses the system double-click time and distance.
- Start/stop and pause shortcuts (`Ctrl+Shift+R`, `Ctrl+Shift+P`), configurable; if another app already uses one at startup, the settings open and mark it.
- Hows does not record its own windows, including keys you press while a Hows window is in front.
- A character typed with `AltGr`, for example `AltGr+Q` for `@` on a German keyboard, counts as typing. Hows does not store it as a shortcut, and the step gets no screenshot.
- A recording shortcut must include `Ctrl`, `Alt`, or `Win`. `Shift` may be added, as in `Ctrl+Shift+R`. `Shift` alone and a key with no modifier are not accepted, and the previous shortcut stays.
- While recording, the main window shrinks to a small recording bar at the top center of the screen. It stays on top, can be dragged, and is left out of every screenshot, including the step screenshots. After stopping, the window returns to its previous size and position.
- If Hows can't receive mouse and keyboard input, recording no longer appears to start. Hows stays in the library or the editor and says what to do in the interface language.
- A step's screenshot is taken around the mouse press. Hows hands the press to its recorder and takes the screenshot right after, so it can already show how the app reacts to the click.

### Editing

- Storyboard editor with a filmstrip: reorder by drag or with Alt+Up and Alt+Down, edit or delete steps, rename the guide.
- Opening a guide loads the screenshot of the selected step, and filmstrip images as they come into view, instead of every screenshot at once.
- Non-destructive annotations: rectangle, arrow, circle, pen, highlighter, blur, and text, with color presets and undo.
- Library with recent guides; open `.steps` files by double-click or from the app.
- Each recent guide shows its first screenshot, title, number of steps, and recording date. Hows reads only the guide data and that one screenshot, so the library stays fast with large files. Moved or deleted files are marked as missing and leave the list when you click them; damaged files are marked as unreadable.
- The annotation toolbar stays at the bottom of the window while the screenshot is taller than the window.
- Before discarding changes or removing all marks, Hows asks in its own dialog in the interface language. Cancel has the focus, so Enter keeps your work.
- A saved guide no longer asks to discard changes when you go back, regardless of where you saved it.
- Unsaved changes are no longer lost without a question. Hows also asks before a new recording replaces the guide, whether it starts from the library, the shortcut, or the notification area, and before **Quit Hows** in the notification area closes the app. Pause, resume, and stop never ask. Leaving a step text unchanged does not count as a change.
- The editor and the exports draw marks from one shared set of defaults: colors, highlighter opacity, pen width, text size and text box, arrowheads, the light halo behind exported text, and the block size of the blur. An exported arrow has the same shape and size as the arrow in the editor, and a mark with a damaged color is drawn in the default mark color.

### Export

- HTML (one self-contained file) and PDF (A4 or US Letter) from the app; Markdown with images and JSON from the command line.
- Guide mode for sharing, bug-report mode with timestamps, element details, and the operating system (CLI).
- The export sheet shows the target folder before exporting and warns when it is synced by OneDrive.
- HTML and PDF exports use the style and accent color from the settings, with numbered step badges and an accent rule under the title. HTML follows the reader's dark mode on screen and prints in the light palette.
- The typeface Atkinson Hyperlegible Next is embedded in HTML and PDF exports, so they look the same without an internet connection. PDFs now show umlauts, accents, and letters such as the Polish ł. An arrow is written as a hyphen and a greater-than sign, because the typeface has no arrow glyph.
- A short “Created with Hows” line in the language of the guide, at the end of HTML and Markdown and in every PDF footer. It is on by default and has no link.
- Saving and exporting never replace an existing file, in the app and on the command line. A taken name gets a number, such as `Guide (2).steps`, and a Markdown export always gets a new folder. A `.steps` file is written to a temporary file first, so a failed save leaves an existing file intact.
- A file name Windows reserves, such as `CON` or `nul.pdf`, gets `_` on that reserved part, so the export can be written.

### Command line

- `steps-cli export`, `demo`, and `from-script` to build guides from JSON without a desktop.
- `steps-cli export` takes `--brand`, `--accent`, and `--no-credit` for the look of HTML, PDF, and Markdown.
- `steps-cli` speaks all seven languages of the app. It uses `--lang`, otherwise the display language of the system, otherwise English. The help names the default style from the style catalog, and `from-script` gives new guides the same language unless the script sets one. Step labels from `from-script` follow the language of the guide, including the French spacing.
- `--format md` works as a short form of `markdown`, and the help and error messages list it.

### File format

- `.steps` schema version 1, documented in [The `.steps` file format](docs/en/file-format.md).
- Opening a `.steps` file checks it against documented [limits](docs/en/file-format.md#limits): step ids use only letters, digits, `-`, and `_` and occur once, sizes and counts stay bounded, and marks stay near the image. A file that breaks a limit is reported as damaged and does not open in part. A Markdown export never writes outside its folder. Exports draw a mark only where it lies inside the image, so a circle that reaches far past the edge of a very wide screenshot no longer stalls the export. A pen line far longer than anyone draws by hand ends early instead of stalling the export.
- Before saving, Hows checks a guide against the same limits, so every `.steps` file it writes opens again in the same version. A guide that breaks a limit is not saved, and an existing file stays intact.
- Two step ids that differ only by ASCII letter case count as the same id, because image names from one guide share a folder on Windows.

### Interface

- Interface in English, German, French, Spanish, Italian, Portuguese, and Dutch; light and dark themes. The German interface uses the informal “du”.
- Plain-language text throughout: no internal terms or text arrows, and every message says what happened and what to do next. A test enforces these rules for every language.
- Step texts and suggested titles use the app's readable name from its version info, for example “Windows PowerShell” instead of “powershell”.
- Step texts, suggested titles, and export labels in English, German, French, Spanish, Italian, Portuguese, and Dutch. Step texts are written as instructions, for example *Click “Save” in “Notepad”*.
- The language setting starts as **Match Windows** and follows the Windows display language; if Hows doesn't have that language, it uses English. A language you choose stays fixed.
- A language change applies at once to the notification area menu and file dialogs, and to step texts from the next recording, without a restart.
- The library and the recording bar show the configured record and pause shortcuts. In the bar, each shortcut sits on its button, which keeps the bar short in every language.
- The library reloads its recent guides each time it opens, so a guide saved a moment ago is listed.
- Screen readers announce the steps in the filmstrip as movable, in the interface language.
- Error messages appear in the interface language and say what to do next, for example when saving, exporting, choosing a folder, or opening a guide from a newer version of Hows. Technical details stay out of the interface. A test checks that every error has a message in every language.
- French texts put a no-break space before `:`, `;`, `!`, and `?` and inside « », including the labels of bug-report exports and the messages of `steps-cli`. The gap is visible in the app, the exports, and the command line. A test checks this for the interface, the export texts, and the command line, and another one checks that the bundled typeface can show every character of every language.

### Design

- Three style presets, Sage (default), Ink, and Ember, each with a light and a dark palette. Every text color meets WCAG AA contrast, checked by a test.
- A custom accent color on top of any preset. The app darkens or lightens it until text on it stays readable.
- The bundled typeface Atkinson Hyperlegible Next, made for legibility, so `Il1` and `0O` look different. No font is loaded from the internet.
- The app no longer contacts Google Fonts at startup. A leftover link in the page asked for the Inter typeface, which the app does not use. A test and a check after every build now fail if the page, its styles, or the built app name a web address that could be loaded, including addresses that start with `//`.
- A Content Security Policy lets the app window load only its bundled scripts, styles, and typeface, the step screenshots, and its own connection to the app, and refuses everything else, including inline scripts and anything from the web.
- Icons instead of text symbols for tools, buttons, and the recording bar.
- A visible focus ring for keyboard use, empty states with a hint for the next step, and errors shown in red with an icon.
- Checkboxes, radio buttons, and scrollbars follow the accent and the theme.
- Spacing, font sizes, and corner radii come from shared scales, so the same step looks the same everywhere. A test flags any new value outside the scales.

### Settings

- PDF paper size (A4 or US Letter) and the color of new marks sit next to the other export and appearance settings.
- Style preset and accent color under Appearance, each with its own reset button.
- A checkbox under Export turns the “Created with Hows” line on or off, with its own reset button.
- An “Advanced” section holds the rarely needed values: the typing and scrolling pauses that start a new step, number of recent guides, PDF page margin, line width of new marks, and templates for guide titles (`{app}`, `{date}`, `{time}`) and file names (`{title}`, `{app}`, `{date}`, `{time}`). The templates show a live example. Dates and times use the time zone of your PC, and `{time}` is written like `14.05` so it works in file names.
- Every changed value has its own reset button, and one button restores all advanced values. Values outside the allowed range are clamped, both in the app and when the settings file is loaded.
- `settings.json` is replaced only after the new file is complete, so a failed save leaves the previous settings usable.
- Numbers in the settings use the decimal mark of the interface language, for example 1.5 in English and 1,5 in German. Both marks are accepted when typing, and the arrow keys step the value.
- All settings controls share one width. A long export folder is shortened by whole folders, for example `C:\…\Documents`, and shows the full path when you point at it.

### Documentation

- User guide in English and German: installation, the SmartScreen warning, recording, editing, export, settings, keyboard shortcuts, privacy, command line, and troubleshooting. README, contributing guide, security policy, and changelog exist in both languages, and every page links to its translation.
- CI checks that every page exists in both languages and that relative links and anchors resolve, and it spell-checks both languages.
- CI checks the documented defaults, ranges, shortcuts, formats, labels, and messages against the code. The pages mark these values, and both languages must mark the same ones.
- The guide now names every error message the app can show, including the one for a file or folder window that does not open, and quotes the startup message for a shortcut that another app already uses. The privacy page says which details a `.steps` file can hold and when they are missing, and the SmartScreen page no longer promises that signed builds stop the warning. After you delete the last step, the previous step is selected, as the app does.
- The privacy page, README, and the export, command line, and file format pages describe what Hows records as the code does. Typed characters are not stored as keyboard input. Guide-mode exports leave out the separate technical fields, but a step text can name a visible button or field and the window or page title. Recordings from the app don't contain the Windows version for now, so a bug report does not always name it.
- The issue form for a wrong step text offers all seven guide languages.

### Build

- CI and release builds run on fixed runner images, Ubuntu 24.04 and Windows Server 2025, so a change of GitHub's default image cannot break a release.
- `THIRD-PARTY-NOTICES.md` lists the typeface, the interface packages, and the Rust crates that ship in the app, with their license texts. The NSIS and MSI installers place that file and `LICENSE` next to `hows.exe`.
- The portable distribution is `Hows_*_x64-portable.zip`, containing `hows.exe`, `LICENSE`, and `THIRD-PARTY-NOTICES.md`. The release workflow does not upload `hows.exe` on its own.
- The MSI ships with `wix-UIExtension-wix3141rtm.zip`, the WiX 3.14.1 UI extension source at commit `b40e9a32c24033e11b77baf2c91a704382f898ed` and its Microsoft Reciprocal License. The NSIS notice names the source archive `nsis-3.11-src.tar.bz2`.
- `THIRD-PARTY-NOTICES.md` also names the packages `npm ls --omit=dev` installs for the interface, including Svelte's compiler dependencies. Those compiler packages are not copied into `hows.exe`.
- Every GitHub Action in the workflows is pinned to a commit SHA.
- Building Hows needs Rust 1.88 or newer, the oldest version the locked dependencies build with.
