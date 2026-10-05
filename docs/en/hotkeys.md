English | [Deutsch](../de/hotkeys.md)

# Keyboard shortcuts

## Recording shortcuts

These shortcuts work everywhere in Windows, even when Hows is in the background.

| Shortcut | Action |
|---|---|
| <!-- fact:settings.hotkey keys -->`Ctrl+Shift+R`<!-- /fact --> | Start a recording, or stop it and open the editor. |
| <!-- fact:settings.pause_hotkey keys -->`Ctrl+Shift+P`<!-- /fact --> | Pause or resume the recording. |

The library and the recording bar always show the shortcuts that are currently set. The same actions are available from the menu of the Hows icon in the notification area.

## Change a shortcut

1. Open [Settings](settings.md#recording).
2. Click the shortcut field. It says <!-- fact:ui.hotkeyListening -->**Press the new shortcut…**<!-- /fact -->.
3. Press the new combination. Press `Esc` to cancel.

Rules:

- A shortcut needs `Ctrl`, `Alt`, or `Win`, plus another key. `Shift` may be added, for example `Ctrl+Shift+F9`. `Shift` alone, such as `Shift+F9`, and a key with no modifier, such as `F9`, are not accepted. Hows shows <!-- fact:ui.errors.hotkey_invalid -->**Hows can't use this shortcut. Choose another one.**<!-- /fact --> and keeps the previous shortcut.
- The two shortcuts must be different. If Windows or another app already uses the combination, Hows shows <!-- fact:ui.hotkeyConflict -->**This shortcut is already taken.**<!-- /fact --> and keeps the old one.
- If a shortcut is already taken when Hows starts, Hows opens the settings and marks it. Click the field and press a new one.
- On German and other keyboard layouts with an `AltGr` key, avoid `Ctrl+Alt+…`. Windows treats `AltGr` as `Ctrl+Alt`, so such a shortcut can collide with typing characters like `@` or `€`.

## Editor

| Key | Where | Action |
|---|---|---|
| `Up` / `Down` | Steps strip | Select the previous or next step. |
| `Home` / `End` | Steps strip | Select the first or last step. |
| `Alt+Up` or `Alt+Left` | Handle of a step | Move the step one place earlier. |
| `Alt+Down` or `Alt+Right` | Handle of a step | Move the step one place later. |
| `Delete` or `Backspace` | Screenshot, while annotating | Remove the selected mark. |
| `Esc` | Screenshot, while annotating | Clear the selection. |
| `Enter` | Text mark | Place the text. |
| `Esc` | Text mark | Cancel the text. |
| `Esc` | Export sheet | Close the sheet. |

## Shortcuts you use while recording

A shortcut with `Ctrl`, `Alt`, or `Win` that you press while recording becomes its own step, for example *Press Ctrl+S*. Everything else you type becomes a typing step without its content. See [Record a guide](recording.md#what-becomes-a-step).

`Alt+F4` quits Hows when a Hows window is in front. Any other close of the recording bar pauses the recording. See [Pause and resume](recording.md#pause-and-resume).

---

[Documentation overview](README.md) · Next: [Privacy](privacy.md)
