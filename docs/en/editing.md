English | [Deutsch](../de/editing.md)

# Edit and annotate

When you stop a recording, the editor opens. At the top, on one line, you see the guide title, <!-- fact:ui.annotateMode -->**Annotate**<!-- /fact -->, and **Export**. Below is the <!-- fact:ui.filmstrip -->**Steps**<!-- /fact --> strip with one card per step. Under that, the step text and <!-- fact:ui.deleteStep -->**Delete step**<!-- /fact --> sit above the screenshot.

## Name the guide

Hows suggests a title from the app you used and the date of the recording, for example *Notepad guide (2026-09-30)*. The title field is selected, so you can type a better name right away. You can change the pattern under [Settings](settings.md#advanced).

## Select and reorder steps

- Click a card in the **Steps** strip to show that step.
- With the strip focused, Up and Down select the previous or next step. Home and End select the first or last.
- To move a step, drag its handle (<!-- fact:ui.gripLabel -->**Move step**<!-- /fact -->) to the new position. With the handle focused, Alt+Up or Alt+Left moves the step one place earlier. Alt+Down or Alt+Right moves it one place later.

## Change the step text

Edit the text in the field above the screenshot. Press `Enter` to keep the text and leave the field. `Shift+Enter` starts a new line. A line under the field says so. Leaving the field another way also keeps the change. A note then says that you changed the text; clear the field to get the generated text back.

## Delete a step

Select the step and click <!-- fact:ui.deleteStep -->**Delete step**<!-- /fact -->. The next step is selected afterward, or the previous one if you deleted the last step.

## Annotate screenshots

Click **Annotate** to draw on the screenshot of the selected step. The toolbar stays at the bottom of the window, even when the screenshot is taller than the window. Click <!-- fact:ui.annotateDone -->**Done**<!-- /fact --> when you are finished.

| Tool | Use it to |
|---|---|
| <!-- fact:ui.tools.rect -->Rectangle<!-- /fact --> | Frame a button or area. |
| <!-- fact:ui.tools.arrow -->Arrow<!-- /fact --> | Point at something. |
| <!-- fact:ui.tools.circle -->Circle<!-- /fact --> | Circle a detail. |
| <!-- fact:ui.tools.pen -->Pen<!-- /fact --> | Draw freehand. |
| <!-- fact:ui.tools.highlight -->Highlighter<!-- /fact --> | Mark an area with translucent color. |
| <!-- fact:ui.tools.blur -->Blur<!-- /fact --> | Make an area unreadable, for example names or numbers. |
| <!-- fact:ui.tools.text -->Text<!-- /fact --> | Add a short label. Press `Enter` to place it, `Esc` to cancel. |

- The <!-- fact:ui.annotateColor -->**Color**<!-- /fact --> button sets the color of the next mark and remembers it. New marks start with <!-- fact:settings.annotation_color -->`#E11D48`<!-- /fact --> until you choose another color. The line width comes from the [advanced settings](settings.md#advanced).
- Click a mark to select it. `Delete` or `Backspace` removes the selected mark; `Esc` clears the selection.
- Click <!-- fact:ui.annotateUndo -->**Undo**<!-- /fact --> to revert the last change. <!-- fact:ui.annotateClear -->**Remove all marks**<!-- /fact --> clears the step after you confirm with <!-- fact:ui.annotateClearAction -->**Remove all**<!-- /fact -->.

Marks never change the original screenshot. Hows stores them separately and draws them only in exports. Blurred areas are pixelated in every export.

## Crop a screenshot

Click **Annotate**, then <!-- fact:ui.crop -->**Crop**<!-- /fact --> on that bar. Drag a corner of the frame. <!-- fact:ui.cropApply -->**Apply crop**<!-- /fact --> keeps that window. `Enter` does the same while the frame is open, and `Esc` cancels. The original picture stays in the file. <!-- fact:ui.cropReset -->**Show full image**<!-- /fact --> brings the hidden edges back. Exports show the window.

## Save your work

Hows does not save automatically. To keep an editable copy, export the guide as a **Hows file (.steps)**; see [Export and share](export.md#hows-file-steps). If you have unsaved changes and go back to the library, open another guide, start a new recording, press `Alt+F4`, or choose <!-- fact:i18n.tray.quit -->**Quit Hows**<!-- /fact --> at the Hows icon in the notification area, Hows first asks whether to discard them. Quitting also asks when a recording already has steps. A recording started with the shortcut or from the notification area asks too. <!-- fact:ui.discardAction -->**Discard**<!-- /fact --> drops the changes; <!-- fact:ui.confirmCancel -->**Cancel**<!-- /fact --> keeps you in the editor. **Cancel** has the focus, so pressing `Enter` keeps your work.

## Open a saved guide

- In the library, click <!-- fact:ui.open -->**Open…**<!-- /fact --> and pick a `.steps` file, or click a guide under <!-- fact:ui.recent -->**Recent guides**<!-- /fact -->. Each card shows the first screenshot, the title, the number of steps, and the recording date.
- Guides whose file was moved or deleted are marked <!-- fact:ui.missing -->**File missing**<!-- /fact -->. When you click one, Hows says so and removes it from the list. Files that aren't readable guides are marked <!-- fact:ui.recentUnreadable -->**Can't be read**<!-- /fact -->.
- If you used an installer, double-click a `.steps` file in File Explorer. While Hows is recording or paused, that shows the recording bar and leaves the recording as it is. If Hows is already open and the guide has unsaved changes, Hows asks before it opens the file.

The guide opens in the editor, and you can change and export it again.

---

[Documentation overview](README.md) · Next: [Export and share](export.md)
