English | [Deutsch](../de/privacy.md)

# Privacy

Hows works without an account and sends no telemetry. After install, the running app uploads nothing and does not open network connections. If Microsoft's WebView2 runtime is missing, the NSIS and MSI installers download it from Microsoft during setup (see [Install Hows](install.md#webview2)). The portable ZIP does not. Hows does not send your guides to a Hows cloud. If you save or export into a folder synced by OneDrive or another sync tool, that tool can copy the file off the PC; the export sheet warns when the export folder is synced.

## Your responsibility when you record or share

Recording starts from the shortcut, the button, or the notification area. Hows has no consent step in the app for other people, and Windows shows no screen-recording permission dialog for these hooks. If you record or share someone else's screen, workplace, call, or chat, that is your responsibility. The same applies when you share an export that shows other people or their data.

## What Hows records

| Recorded | Not recorded |
|---|---|
| Clicks, double-clicks, and right-clicks, with the position on screen | Typed characters as keyboard input |
| A screenshot of the monitor at each click, shortcut, and scroll step | Screenshots of typing steps |
| Shortcuts with `Ctrl`, `Alt`, or `Win`, for example `Ctrl+S` | Single keys, `Shift` combinations, and characters typed with `AltGr`, such as `@`, which only count as typing |
| That you typed and in which field | The windows of Hows itself, including keys you press while a Hows window is in front |
| The name and type of the clicked element, window title, and app name | Anything while the recording is paused |
| The name of the operating system, but not the Windows version | |

Hows does not store the characters you type as keyboard input. A typing step gets no screenshot. The screenshot of a later step can still show text you typed, for example in a field that is still open.

## What stays on your PC

- **`.steps` files** in your guides folder hold your steps, usually each with its original screenshot, and the details Windows provided for it: the window title, the app name, and the element that was clicked. Window titles can contain document or customer names.
- **Exports** go to your export folder.
- **Settings** are stored in a <!-- fact:app.settings_file -->`settings.json`<!-- /fact --> file in the <!-- fact:app.identifier -->`how.hows`<!-- /fact --> folder under `%APPDATA%`, including the list of recent guides.

Nothing leaves your PC through Hows unless you copy or share a file yourself. A synced folder can still upload a file you put there.

## License of Hows, not of what you capture

The [MIT license](../../LICENSE) covers Hows itself. It does not cover the content of your screenshots or step texts. Full-monitor screenshots and window or page titles can show other people's software, documents, chats, or web pages. Publishing a guide may require rights you hold separately from the Hows license.

## Exports

Exports from the app contain the guide only: the title, the step texts, and the screenshots with your marks. They leave out the separate technical fields: element details, app name, window title, click position, timestamp, monitor, and operating system.

The step texts Hows writes name what you clicked and where, for example *<!-- fact:i18n.step.click.target_in_context -->Click “{target}” in “{context}”<!-- /fact -->*. So a step text can contain the visible name of a button or field and the window title, which in a browser is the page title. Window and page titles can contain document, customer, or email names. Read the step texts as well as the screenshots before you share, and change a text in the editor if needed.

HTML and PDF exports also carry the line “<!-- fact:i18n.export.credit -->Created with Hows<!-- /fact -->” unless you turn it off in [Settings](settings.md#export). The line names the app only. It has no link, no tracking, and no data about you or your PC.

The bug-report mode of `steps-cli` adds window titles, element details, click positions, timestamps, monitor details, and the operating system. It adds the Windows version only when the `.steps` file contains one. Recordings from the app don't contain it for now, so a bug report does not always name the Windows version. Use bug-report mode only when the recipient needs these details. See [Command line](cli.md#guide-mode-and-bug-report-mode).

## Before you share

- Screenshots show whatever was on screen, including notifications, other windows, and text you typed, so they can contain private data. Check every step.
- Use the <!-- fact:ui.tools.blur -->**Blur**<!-- /fact --> tool on names, addresses, numbers, and anything else that is private. Blurred areas are pixelated in every export; the original screenshot inside the `.steps` file stays unchanged.
- A `.steps` file contains the original, unmarked screenshots and the details Windows provided, including window titles. Share an HTML or PDF export instead if the recipient does not need to edit the guide.
- If your export folder is synced by OneDrive, the export sheet warns you before exporting. See [Export and share](export.md#onedrive-warning).

## Report a problem

If Hows ever records typed text, writes files outside the folders you chose (other than the default folders it uses when yours can't be used), or opens a network connection, please report it privately as described in the [security policy](../../SECURITY.md).

---

[Documentation overview](README.md) · Next: [Command line](cli.md)
