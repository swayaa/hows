English | [Deutsch](../de/architecture.md)

# How Hows fits together

Hows records what you click and turns it into an illustrated guide. The guide stays on your PC until you save or share it. This page shows which parts take part. Module names in the pictures are the names from the program, so the picture stays true.

The pictures are in German. The Light, Classic, Present, and Export buttons in the diagram stay English. The runtime picture's legend also uses the English words Frontend, Backend, and External.

Hows runs on Windows 10 and Windows 11. See [Install Hows](install.md).

## The five pictures

- [Runtime](../de/diagrams/runtime.html): the window, the recording bar, commands, the recorder, and the libraries.
- [Recording states](../de/diagrams/lifecycle.html): Idle, Recording, Paused, and Reviewing. Discard returns to Idle.
- [From a click to a step](../de/diagrams/click-to-step.html): from the hook, through the event, to the image, the element, and the step.
- [From recording to export](../de/diagrams/recording-to-export.html): start, pause, stop, the editor, then save or export.
- [What happens to the image](../de/diagrams/screenshot-copies.html): the `.steps` file keeps the original PNG. HTML, PDF, and Markdown get the cropped copy with the marks.

## What runs at the same time

The interface is a WebView inside the Tauri window. While Hows records, that same window becomes the recording bar. The Tauri shell starts the recorder thread. Commands from the interface start, pause, and stop the recording.

steps-session builds the steps in memory. steps-capture supplies the image first and then the element. steps-i18n writes the sentence. steps-store writes the `.steps` file. steps-export produces HTML, PDF, and Markdown. steps-cli is a separate program without a window. It reads the file through steps-store and writes through steps-export. Settings live in `settings.json` on the PC.

## From a recording to a file

A recording is ready (Idle), running (Recording), paused (Paused), or open in the editor (Reviewing). Discard in the editor clears the steps and returns to Idle. Pause is optional. Stop from the recording or from the pause opens the editor. Hows does not save by itself. Save writes the `.steps` file. From the editor you export HTML or PDF.

Crop sits on the annotate bar and can be undone. See [Crop a screenshot](editing.md#crop-a-screenshot).

How you close the bar, what a second start does, and how a scroll step is worded are in [Record a guide](recording.md) and [Open a saved guide](editing.md#open-a-saved-guide).

## The image and the license

The `.steps` file keeps the captured PNG whole. HTML, PDF, and Markdown show the cropped copy, with the marks drawn into that copy.

The [MIT license](../../LICENSE) covers Hows, not the screenshots. The person recording is responsible for other people's screens and for sharing. The running app opens no network connection. The installers may download WebView2. The portable ZIP does not. See [Privacy](privacy.md) and [WebView2](install.md#webview2).

---

[Documentation overview](README.md)
