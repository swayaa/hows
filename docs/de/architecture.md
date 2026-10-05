[English](../en/architecture.md) | Deutsch

# Wie Hows aufgebaut ist

Hows nimmt auf, was du anklickst, und macht daraus eine bebilderte Anleitung. Die Anleitung bleibt auf deinem PC, bis du sie speicherst oder teilst. Diese Seite zeigt, welche Teile dabei mitspielen. Die Modulnamen in den Bildern sind die Namen aus dem Programm, damit das Bild stimmt.

Die Zeichnungen sind auf Deutsch. Die Schaltflächen Light, Classic, Present und Export im Diagramm bleiben Englisch. In der Legende des Laufzeitbildes stehen außerdem die englischen Wörter Frontend, Backend und External.

Hows läuft auf Windows 10 und Windows 11. Siehe [Hows installieren](install.md).

## Die fünf Bilder

- [Hows zur Laufzeit](diagrams/runtime.html): Fenster, Aufnahmeleiste, Befehle, Recorder und die Bibliotheken.
- [Zustände einer Aufnahme](diagrams/lifecycle.html): Idle, Recording, Paused und Reviewing. Verwerfen führt zurück nach Idle.
- [Vom Klick zum Schritt](diagrams/click-to-step.html): vom Hook über das Ereignis zu Bild, Element und Schritt.
- [Von der Aufnahme zum Export](diagrams/recording-to-export.html): Start, Pause, Stopp, Editor, Speichern oder Export.
- [Was mit dem Bild passiert](diagrams/screenshot-copies.html): die `.steps`-Datei behält das Original-PNG. HTML, PDF und Markdown bekommen die zugeschnittene Kopie mit den Markierungen.

## Was gleichzeitig läuft

Die Oberfläche ist eine WebView im Tauri-Fenster. Während der Aufnahme wird dasselbe Fenster zur Aufnahmeleiste. Die Tauri-Shell startet den Recorder-Thread. Befehle aus der Oberfläche starten, pausieren und beenden die Aufnahme.

steps-session baut die Schritte im Speicher. steps-capture liefert zuerst das Bild und danach das Element. steps-i18n schreibt den Satz. steps-store schreibt die `.steps`-Datei. steps-export erzeugt HTML, PDF und Markdown. steps-cli ist ein eigenes Programm ohne Fenster. Es liest die Datei über steps-store und schreibt über steps-export. Die Einstellungen liegen in `settings.json` auf dem PC.

## Von der Aufnahme bis zur Datei

Eine Aufnahme ist bereit (Idle), läuft (Recording), pausiert (Paused) oder liegt im Editor (Reviewing). Verwerfen im Editor leert die Schritte und kehrt nach Idle zurück. Pause ist freiwillig. Stopp aus der Aufnahme oder aus der Pause öffnet den Editor. Hows speichert nicht von selbst. Speichern schreibt die `.steps`-Datei. Aus dem Editor exportierst du HTML oder PDF.

Zuschneiden sitzt in der Leiste von **Markieren** und lässt sich zurücknehmen. Siehe [Einen Screenshot zuschneiden](editing.md#einen-screenshot-zuschneiden).

Wie du die Leiste schließt, was ein zweiter Start tut und wie ein Scroll-Schritt lautet, steht in [Eine Anleitung aufnehmen](recording.md) und beim [Öffnen einer gespeicherten Anleitung](editing.md#eine-gespeicherte-anleitung-öffnen).

## Das Bild und die Lizenz

In der `.steps`-Datei bleibt das aufgenommene PNG vollständig. HTML, PDF und Markdown zeigen die zugeschnittene Kopie, in die Hows die Markierungen zeichnet.

Die [MIT-Lizenz](../../LICENSE) gilt für Hows, nicht für die Screenshots. Wer aufnimmt, ist verantwortlich für fremde Bildschirme und für das Teilen. Die laufende App baut keine Netzwerkverbindung auf. Die Installer können WebView2 herunterladen, das portable ZIP nicht. Siehe [Datenschutz](privacy.md) und [WebView2](install.md#webview2).

---

[Dokumentationsüberblick](README.md)
