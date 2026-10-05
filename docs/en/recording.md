English | [Deutsch](../de/recording.md)

# Record a guide

## Start a recording

You have three ways to start:

- Click <!-- fact:ui.newCapture -->**New recording**<!-- /fact --> in the library.
- Press the record shortcut, <!-- fact:settings.hotkey keys -->`Ctrl+Shift+R`<!-- /fact --> by default.
- Click the Hows icon in the notification area and choose <!-- fact:i18n.tray.toggle -->**Start or stop recording**<!-- /fact -->.

Then work as usual. While Hows records, its window shrinks to a small recording bar at the top center of the screen. The bar shows a pulsing red dot, the number of steps so far, and the **Pause** and **Stop** buttons, each with its shortcut. While you pause, the bar says <!-- fact:ui.capturePaused -->**Paused**<!-- /fact -->. It stays on top of other windows, and you can drag it elsewhere. Hows never records its own windows. Clicking the bar adds no step, and neither do keys you press while a Hows window is in front. The bar never appears in a screenshot. When you stop, the window returns to its previous size and position.

Windows leaves the bar out of every screen capture while you record. That includes your own screenshots, screen recordings, and screen sharing, for example in a video call. Other people in the call don't see the bar, but you still do.

## What becomes a step

| What you do | Resulting step | Screenshot |
|---|---|---|
| Click, double-click, or right-click | One step each, for example *Click “Save” in “Notepad”* | Yes |
| Press a shortcut with `Ctrl`, `Alt`, or `Win`, for example `Ctrl+S` | One step, for example *Press Ctrl+S* | Yes |
| Scroll with the mouse wheel or touchpad | One step for each window and scroll direction, for example *Scroll down 3 in “Documents”* | Yes |
| Type text | One step per field until you pause, for example *Type your text in “Search”* | No |

Details:

- **Double-click** uses the double-click speed and distance set in Windows.
- **Scrolling** in the same window and direction stays one step until you stop scrolling for longer than the scroll pause. The pause length is a setting (<!-- fact:settings.scroll_pause_ms seconds -->1.5<!-- /fact --> seconds by default). The sentence names that window and how many wheel notches you scrolled. Further movement in the pause only adds to that number. Hows inspects the control under the pointer when the step starts, not on every further movement.
- **Typing** stays one step until you pause for longer than the typing pause (<!-- fact:settings.typing_pause_ms seconds -->3<!-- /fact --> seconds by default). Hows never stores what you type, only that you typed and where. Typing steps get no screenshot, because a screenshot would show the text. The click that selected the field usually shows it.
- `Shift` alone does not make a shortcut. `Shift+A` counts as typing. So does a character you type with `AltGr`, for example `AltGr+Q` for `@` on a German keyboard.
- Each screenshot shows the whole monitor under the mouse pointer.

Click, right-click, double-click, and typing texts name the control and, when Hows knows it, the window. A scroll text names the window and the number of wheel notches. Hows writes them in the language set under [Settings](settings.md#appearance) when the recording starts.

## Pause and resume

Click **Pause** in the bar, press the pause shortcut (<!-- fact:settings.pause_hotkey keys -->`Ctrl+Shift+P`<!-- /fact --> by default), or choose <!-- fact:i18n.tray.pause -->**Pause or resume**<!-- /fact --> from the notification area icon. While paused, Hows records nothing. Click **Resume** or press the shortcut again to continue.

`Alt+F4` quits Hows. Any other close of the recording bar, for example from the taskbar, pauses the recording and hides the bar. Hows keeps running and records nothing while the bar is hidden. If the recording was already paused, it stays paused. If quitting would drop steps already recorded, or unsaved work in the editor, Hows asks first. See [Save your work](editing.md#save-your-work).

## Stop

Click **Stop**, press the record shortcut again, or choose **Start or stop recording** from the notification area icon. The editor opens with all steps. Continue with [Edit and annotate](editing.md).

If you recorded no steps, the editor says so and offers a new recording.

## Tips for clean guides

- Close windows that show private data before you start, or pause while they are on screen.
- Move slowly enough that each click hits the element you mean.
- Screenshots show whatever is on screen. Check them before you share, and use the blur tool for sensitive areas. See [Privacy](privacy.md).
- Hows does not ask other people for consent before recording, and Windows shows no screen-recording dialog for these hooks. Recording or sharing someone else's screen, workplace, call, or chat is your responsibility. See [Your responsibility when you record or share](privacy.md#your-responsibility-when-you-record-or-share).

## Limits

- Recording works on Windows only.
- Apps with little accessibility information (some games, remote desktops, custom-drawn interfaces) get generic step texts that name only the window, not the button. You can rewrite them in the editor.

---

[Documentation overview](README.md) · Next: [Edit and annotate](editing.md)
