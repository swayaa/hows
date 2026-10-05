[English](../en/hotkeys.md) | Deutsch

# Tastenkürzel

## Kürzel für die Aufnahme

Diese Kürzel funktionieren überall in Windows, auch wenn Hows im Hintergrund ist.

| Kürzel | Aktion |
|---|---|
| <!-- fact:settings.hotkey keys -->`Strg+Umschalt+R`<!-- /fact --> | Eine Aufnahme starten, oder sie beenden und den Editor öffnen. |
| <!-- fact:settings.pause_hotkey keys -->`Strg+Umschalt+P`<!-- /fact --> | Die Aufnahme pausieren oder fortsetzen. |

Die Bibliothek und die Aufnahmeleiste zeigen immer die Kürzel, die gerade eingestellt sind. Dieselben Aktionen findest du im Menü des Hows-Symbols im Infobereich.

## Ein Tastenkürzel ändern

1. Öffne die [Einstellungen](settings.md#aufnahme).
2. Klick in das Feld des Kürzels. Dort steht <!-- fact:ui.hotkeyListening -->**Drück das neue Tastenkürzel…**<!-- /fact -->.
3. Drück die neue Kombination. Mit `Esc` brichst du ab.

Regeln:

- Ein Kürzel braucht `Strg`, `Alt` oder `Win` und eine weitere Taste. `Umschalt` darf dazu, zum Beispiel `Strg+Umschalt+F9`. `Umschalt` allein, etwa `Umschalt+F9`, und eine Taste ohne Modifikator, etwa `F9`, gelten nicht. Hows zeigt <!-- fact:ui.errors.hotkey_invalid -->**Dieses Tastenkürzel kann Hows nicht nutzen. Wähl ein anderes.**<!-- /fact --> und behält das bisherige Kürzel.
- Die beiden Kürzel müssen sich unterscheiden. Nutzt Windows oder eine andere App die Kombination schon, zeigt Hows <!-- fact:ui.hotkeyConflict -->**Dieses Tastenkürzel ist schon vergeben.**<!-- /fact --> und behält das alte Kürzel.
- Ist ein Kürzel schon vergeben, wenn Hows startet, öffnet Hows die Einstellungen und markiert es. Klick ins Feld und drück ein neues.
- Auf deutschen und anderen Tastaturen mit `AltGr` meidest du besser `Strg+Alt+…`. Windows behandelt `AltGr` wie `Strg+Alt`, deshalb kann so ein Kürzel mit Zeichen wie `@` oder `€` kollidieren.

## Editor

| Taste | Wo | Aktion |
|---|---|---|
| `Pfeil nach oben` / `Pfeil nach unten` | Streifen Schritte | Vorigen oder nächsten Schritt wählen. |
| `Pos1` / `Ende` | Streifen Schritte | Ersten oder letzten Schritt wählen. |
| `Alt+Pfeil nach oben` oder `Alt+Pfeil nach links` | Griff eines Schritts | Schritt eine Stelle nach vorn schieben. |
| `Alt+Pfeil nach unten` oder `Alt+Pfeil nach rechts` | Griff eines Schritts | Schritt eine Stelle nach hinten schieben. |
| `Entf` oder `Rücktaste` | Screenshot, beim Markieren | Ausgewählte Markierung entfernen. |
| `Esc` | Screenshot, beim Markieren | Auswahl aufheben. |
| `Enter` | Textmarkierung | Text setzen. |
| `Esc` | Textmarkierung | Text verwerfen. |
| `Esc` | Exportblatt | Blatt schließen. |

## Kürzel, die du während der Aufnahme drückst

Ein Kürzel mit `Strg`, `Alt` oder `Win`, das du während der Aufnahme drückst, wird ein eigener Schritt, zum Beispiel *Drücke Strg+S*. Alles andere, was du tippst, wird ein Tipp-Schritt ohne Inhalt. Siehe [Eine Anleitung aufnehmen](recording.md#was-zu-einem-schritt-wird).

`Alt+F4` beendet Hows, wenn ein Fenster von Hows vorne ist. Jedes andere Schließen der Aufnahmeleiste pausiert die Aufnahme. Siehe [Pausieren und fortsetzen](recording.md#pausieren-und-fortsetzen).

---

[Zur Übersicht](README.md) · Weiter: [Datenschutz](privacy.md)
