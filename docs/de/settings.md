[English](../en/settings.md) | Deutsch

# Einstellungen

Klick in der Bibliothek auf **Einstellungen**. Die Einstellungen werden auf deinem PC gespeichert, und Änderungen gelten sofort. Änderst du einen Wert mit Standard, etwa das Papierformat, den Stil oder einen Wert unter **Erweitert**, erscheint daneben <!-- fact:ui.resetValue -->**Zurücksetzen**<!-- /fact --> und stellt den Standard wieder her.

## Aufnahme

| Einstellung | Standard | Was sie tut |
|---|---|---|
| <!-- fact:ui.hotkeyRecord -->Aufnahme starten oder beenden<!-- /fact --> | <!-- fact:settings.hotkey keys -->`Strg+Umschalt+R`<!-- /fact --> | Globales Tastenkürzel, das eine Aufnahme startet oder sie beendet und den Editor öffnet. |
| <!-- fact:ui.hotkeyPause -->Pausieren oder fortsetzen<!-- /fact --> | <!-- fact:settings.pause_hotkey keys -->`Strg+Umschalt+P`<!-- /fact --> | Globales Tastenkürzel, das eine laufende Aufnahme pausiert oder fortsetzt. |

Um ein Kürzel zu ändern, klick in sein Feld und drück die neue Kombination. Die Regeln stehen unter [Tastenkürzel](hotkeys.md#ein-tastenkürzel-ändern).

## Export

| Einstellung | Standard | Was sie tut |
|---|---|---|
| <!-- fact:ui.exportDefault -->Standardformat<!-- /fact --> | <!-- fact:settings.export_format ui:exportFormat -->HTML<!-- /fact --> | Format, mit dem das Exportblatt startet: <!-- fact:app.export_formats ui:exportFormat -->HTML, PDF oder `.steps`<!-- /fact -->. |
| <!-- fact:ui.openAfterExport -->Nach dem Export öffnen<!-- /fact --> | <!-- fact:settings.open_after_export bool -->An<!-- /fact --> | Zeigt die exportierte Datei im Datei-Explorer. |
| <!-- fact:ui.exportCredit -->„Erstellt mit Hows“ in Exporten zeigen<!-- /fact --> | <!-- fact:settings.export_credit bool -->An<!-- /fact --> | Ergänzt HTML- und PDF-Exporte um eine kurze Zeile mit dem Hinweis auf Hows. Siehe [Export](export.md#aussehen-und-hinweiszeile). |
| <!-- fact:ui.pdfPaper -->Papierformat für PDF<!-- /fact --> | <!-- fact:settings.pdf_paper ui:paperNames -->A4<!-- /fact --> | <!-- fact:export.papers ui:paperNames -->A4 oder US Letter<!-- /fact -->. |
| <!-- fact:ui.defaultFolder -->Exportordner<!-- /fact --> | Dokumente | Wohin HTML- und PDF-Exporte gehen. **Durchsuchen…** wählt einen anderen Ordner. |

## Darstellung

| Einstellung | Standard | Was sie tut |
|---|---|---|
| <!-- fact:ui.language -->Sprache<!-- /fact --> | <!-- fact:settings.language ui:language -->Wie Windows<!-- /fact --> | Sprache der Oberfläche, des Menüs im Infobereich, der Dateidialoge und neuer Schritttexte. <!-- fact:ui.languageSystem -->**Wie Windows**<!-- /fact --> nutzt die Windows-Anzeigesprache, oder Englisch, wenn Hows sie nicht anbietet. Verfügbar: <!-- fact:i18n.languages -->English, Deutsch, Français, Español, Italiano, Português (Brasil), Nederlands<!-- /fact -->. |
| <!-- fact:ui.theme -->Farbschema<!-- /fact --> | <!-- fact:settings.theme ui:theme -->Wie Windows<!-- /fact --> | Hell, Dunkel oder wie Windows. |
| <!-- fact:ui.brand -->Stil<!-- /fact --> | <!-- fact:settings.brand ui:brandNames -->Salbei<!-- /fact --> | Farbvorlage der App: <!-- fact:brands.codes ui:brandNames -->Salbei, Tinte oder Glut<!-- /fact -->, jeweils mit heller und dunkler Palette. |
| <!-- fact:ui.accentColor -->Akzentfarbe<!-- /fact --> | Der Akzent des Stils | Deine eigene Akzentfarbe. Hows dunkelt sie ab oder hellt sie auf, bis Text darauf lesbar bleibt. |

Ein Sprachwechsel gilt sofort für die Oberfläche, das Menü im Infobereich und die Dateidialoge. Schritttexte schon aufgenommener Anleitungen bleiben, wie sie sind; die neue Sprache gilt ab der nächsten Aufnahme. Mit **Wie Windows** folgt Hows einer neuen Windows-Anzeigesprache, sobald du es neu startest.

Stil und Akzentfarbe gelten auch für HTML- und PDF-Exporte. Siehe [Export](export.md#aussehen-und-hinweiszeile).

## Ordner

| Einstellung | Standard | Was sie tut |
|---|---|---|
| <!-- fact:ui.guidesFolder -->Anleitungsordner<!-- /fact --> | <!-- fact:app.guides_subfolder -->`Steps`<!-- /fact --> in deinem Ordner Dokumente | Wo `.steps`-Dateien gespeichert werden. **Durchsuchen…** wählt einen anderen Ordner. |

Kann Hows einen Ordner nicht nutzen, speichert es stattdessen im Standardordner und zeigt einen Hinweis.

## Erweitert

Öffne <!-- fact:ui.groupAdvanced -->**Erweitert**<!-- /fact --> für Werte, die du selten brauchst. Jeder startet mit einem sinnvollen Standard; <!-- fact:ui.resetAdvanced -->**Alle erweiterten Einstellungen zurücksetzen**<!-- /fact --> stellt alle wieder her.

| Einstellung | Standard | Bereich | Was sie tut |
|---|---|---|---|
| <!-- fact:ui.typingPause -->Tipp-Pause bis zum neuen Schritt<!-- /fact --> | <!-- fact:settings.typing_pause_ms seconds -->3<!-- /fact --> s | <!-- fact:limits.typing_pause_ms.min seconds -->0,5<!-- /fact --> bis <!-- fact:limits.typing_pause_ms.max seconds -->10<!-- /fact --> s | Eine längere Pause beim Tippen startet einen neuen Tipp-Schritt. |
| <!-- fact:ui.scrollPause -->Scroll-Pause bis zum neuen Schritt<!-- /fact --> | <!-- fact:settings.scroll_pause_ms seconds -->1,5<!-- /fact --> s | <!-- fact:limits.scroll_pause_ms.min seconds -->0,25<!-- /fact --> bis <!-- fact:limits.scroll_pause_ms.max seconds -->5<!-- /fact --> s | Eine längere Pause beim Scrollen startet einen neuen Scroll-Schritt. |
| <!-- fact:ui.recentLimit -->Zuletzt geöffnete Anleitungen in der Bibliothek<!-- /fact --> | <!-- fact:settings.recent_limit -->8<!-- /fact --> | <!-- fact:limits.recent_limit.min -->1<!-- /fact --> bis <!-- fact:limits.recent_limit.max -->30<!-- /fact --> | Wie viele Anleitungen die Bibliothek zeigt. |
| <!-- fact:ui.titleTemplate -->Titel neuer Aufnahmen<!-- /fact --> | Leer (der Standard deiner Sprache) | | Muster für den vorgeschlagenen Titel. `{app}` wird zur benutzten App, `{date}` zum Tag und `{time}` zur Uhrzeit der Aufnahme. |
| <!-- fact:ui.fileNameTemplate -->Dateiname<!-- /fact --> | <!-- fact:settings.file_name_template -->`{title}`<!-- /fact --> | | Muster für Dateinamen von Exporten. `{title}` wird zum Titel der Anleitung, `{app}` zur benutzten App, `{date}` zum Tag und `{time}` zur Uhrzeit der Aufnahme. |
| <!-- fact:ui.pdfMargin -->Seitenrand für PDF<!-- /fact --> | <!-- fact:settings.pdf_margin_mm -->18<!-- /fact --> mm | <!-- fact:limits.pdf_margin_mm.min -->5<!-- /fact --> bis <!-- fact:limits.pdf_margin_mm.max -->40<!-- /fact --> mm | Rand um jede PDF-Seite. |
| <!-- fact:ui.annotationStroke -->Linienstärke neuer Markierungen<!-- /fact --> | <!-- fact:settings.annotation_stroke -->2,5<!-- /fact --> px | <!-- fact:limits.annotation_stroke.min -->1<!-- /fact --> bis <!-- fact:limits.annotation_stroke.max -->8<!-- /fact --> px | Start-Linienstärke neuer Markierungen. |

Beide Muster zeigen unter dem Feld sofort ein Beispiel. Datum und Uhrzeit folgen der Zeitzone deines PCs. `{date}` erscheint als Jahr, Monat und Tag, zum Beispiel `2026-09-30`. `{time}` erscheint als Stunden und Minuten im 24-Stunden-Format, zum Beispiel `14.05`; der Punkt steht dort, weil Dateinamen keinen Doppelpunkt enthalten dürfen. Das Muster für Dateinamen kann keine Ordner anlegen: `/` und `\` werden zu Leerzeichen.

Zahlen nutzen das Dezimalzeichen der Oberflächensprache, zum Beispiel 1,5 auf Deutsch und 1.5 auf Englisch. Du kannst beide Zeichen tippen. Steht der Fokus im Feld, erhöhen oder verringern `Pfeil nach oben` und `Pfeil nach unten` den Wert um einen Schritt.

Werte außerhalb des Bereichs setzt Hows auf den nächsten erlaubten Wert. Das gilt auch, wenn du die Einstellungsdatei von Hand bearbeitest.

## Wo die Einstellungen liegen

Hows speichert seine Einstellungen in einer Datei <!-- fact:app.settings_file -->`settings.json`<!-- /fact --> in seinem Ordner unter `%APPDATA%`. Sie enthält die Werte dieser Seite und die Liste der zuletzt geöffneten Anleitungen. Siehe [Datenschutz](privacy.md#was-auf-deinem-pc-bleibt).

---

[Zur Übersicht](README.md) · Weiter: [Tastenkürzel](hotkeys.md)
