English | [Deutsch](../de/troubleshooting.md)

# Troubleshooting

<!-- fact-exempt: ui.errors.hotkey_invalid ui.errors.hotkey_taken ui.errors.hotkey_conflict ui.errors.invalid_folder ui.errors.path_missing ui.errors.reveal_failed ui.errors.window_failed -->

## Windows protected your PC

Hows is not code-signed yet. See [The SmartScreen warning](smartscreen.md).

## The portable app does not open

`hows.exe` inside the portable ZIP needs the Microsoft Edge WebView2 runtime. Windows 11 includes it. On Windows 10, install the WebView2 runtime from Microsoft, or use one of the installers, which add it for you. See [Install Hows](install.md#webview2).

## Shortcut not available

Another app already uses the record or pause shortcut. When Hows starts, it shows <!-- fact:ui.hotkeyRegisterError -->**Shortcut not available**<!-- /fact --> with the shortcut, opens the settings, and marks the shortcut with <!-- fact:ui.hotkeyTakenAtStartup -->**Another app already uses this shortcut. Click the field and press a new one.**<!-- /fact --> Pick a combination that no other app uses. See [Change a shortcut](hotkeys.md#change-a-shortcut).

The message <!-- fact:ui.hotkeyConflict -->**This shortcut is already taken.**<!-- /fact --> appears when you pick a combination that Windows, another app, or the other Hows shortcut already uses. Hows keeps the previous shortcut.

## Recording does not start

The message <!-- fact:ui.errors.input_hook_failed -->**Recording didn't start because Hows can't receive mouse and keyboard input. Restart Hows and try again. If it happens again, check whether security software is blocking Hows.**<!-- /fact --> means Windows did not let Hows listen for clicks and keys. Hows stays in the library or the editor and records nothing. Restart Hows. If the message comes back, ask whoever manages the security software on your PC to allow Hows.

## No steps were recorded

The recording ended without a single step. Check that you did not pause the recording, then start a new one. Clicks on the windows of Hows itself never become steps.

## A step has generic step text or no screenshot

- Generic step texts such as *Click in “…”* come from apps that give Windows little information about their buttons. Rewrite the text in the editor.
- Typing steps never have a screenshot, so your input stays private. The step before usually shows the field.
- If a screenshot could not be taken, the step is still kept and shows <!-- fact:ui.noScreenshot -->**This step has no screenshot.**<!-- /fact -->

## A guide does not open

| Message | Meaning |
|---|---|
| <!-- fact:ui.errors.not_found -->**File not found**<!-- /fact --> | The file was moved, renamed, or deleted. The library marks such entries <!-- fact:ui.missing -->**File missing**<!-- /fact -->. |
| <!-- fact:ui.errors.not_steps -->**This isn't a Hows file (.steps).**<!-- /fact --> | The file is not a `.steps` file. |
| <!-- fact:ui.errors.corrupt -->**This guide can't be opened. The file may be damaged.**<!-- /fact --> | The file is incomplete or not a valid `.steps` archive. |
| <!-- fact:ui.errors.newer_version -->**This guide comes from a newer version of Hows. Update Hows to open it.**<!-- /fact --> | The file uses a newer [file format](file-format.md) than your version of Hows supports. |

## Saving or exporting fails

| Message | What to do |
|---|---|
| <!-- fact:ui.errors.save_failed -->**The guide couldn't be saved. Check that you can write to the guides folder.**<!-- /fact --> | Pick another guides folder under [Settings](settings.md#folders), or free up space on the drive. |
| <!-- fact:ui.errors.export_failed -->**The export couldn't be written. Check that you can write to the export folder and that the file isn't open in another app.**<!-- /fact --> | Close the previous export, for example in a PDF viewer, then export again. |
| <!-- fact:ui.errors.folder_unavailable -->**Hows can't create or open the folder. Choose another folder in Settings.**<!-- /fact --> | The folder is on a drive that is gone or read-only. Pick a local folder. |
| <!-- fact:ui.errors.settings_not_saved -->**Your settings couldn't be saved. Check that you can write to your user folder.**<!-- /fact --> | Check the permissions of the <!-- fact:app.identifier -->`how.hows`<!-- /fact --> folder under `%APPDATA%`, then change the setting again. |

The message <!-- fact:ui.errors.internal -->**Something went wrong inside Hows. Restart Hows and try again.**<!-- /fact --> means Hows hit an error it doesn't expect. If it happens again, report it as described below.

## A file or folder window does not open

The message <!-- fact:ui.errors.dialog_failed -->**The window for choosing a file or folder didn't open. Try again.**<!-- /fact --> appears when Windows couldn't open the window for **Open…**, **Browse…**, or **Change…**. Click the button again. If the window still doesn't open, restart Hows.

## Exports land in the wrong place

- The note <!-- fact:ui.pathInvalid -->**Hows can't use this folder, so it saves to the default folder instead.**<!-- /fact --> means the chosen folder is missing or not writable. Pick another one in the export sheet or under [Settings](settings.md#export).
- The warning <!-- fact:ui.exportSyncedWarning -->**OneDrive syncs this folder, so the screenshots will end up in the cloud.**<!-- /fact --> appears before you export to a synced folder. Pick a local folder with <!-- fact:ui.exportChangeFolder -->**Change…**<!-- /fact --> if the guide must not leave your PC.
- A file name ending in a number, such as `Guide (2).html`, means a file with that name already existed. Hows never replaces it. See [Folders and file names](export.md#folders-and-file-names).

## Nothing to export yet

The message <!-- fact:ui.noSessionExport -->**Nothing to export yet. Record a guide first.**<!-- /fact --> or <!-- fact:ui.errors.no_guide -->**No guide is open. Record or open one first.**<!-- /fact --> appears when no guide is open in the editor. Record one or open a `.steps` file.

## Still stuck?

Open an issue using the bug template and follow the tips in [Contributing](../../CONTRIBUTING.md#reporting). For security problems, follow the [security policy](../../SECURITY.md) instead.

---

[Documentation overview](README.md)
