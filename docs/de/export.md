[English](../en/export.md) | Deutsch

# Exportieren und teilen

Klick im Editor auf <!-- fact:ui.exportPrimary -->**Exportieren**<!-- /fact -->. Das Blatt <!-- fact:ui.exportTitle -->**Anleitung exportieren**<!-- /fact --> öffnet sich:

1. Wähl ein Format: <!-- fact:ui.exportHtml -->**Webseite (HTML)**<!-- /fact -->, <!-- fact:ui.exportPdf -->**PDF-Dokument**<!-- /fact --> oder <!-- fact:ui.exportSteps -->**Hows-Datei (.steps)**<!-- /fact -->. Das Blatt startet mit deinem Standardformat aus den [Einstellungen](settings.md#export).
2. Prüfe den Ordner neben <!-- fact:ui.exportTarget -->**Speichert in**<!-- /fact -->. <!-- fact:ui.exportChangeFolder -->**Ändern…**<!-- /fact --> wählt einen anderen und merkt ihn sich.
3. Lass <!-- fact:ui.exportOpenAfter -->**Danach öffnen**<!-- /fact --> angehakt, wenn der Datei-Explorer die Datei danach zeigen soll.
4. Klick auf **Exportieren**. Eine Meldung zeigt, wo die Datei gespeichert wurde.

Mit `Esc` oder <!-- fact:ui.closeSheet -->**Schließen**<!-- /fact --> verlässt du das Blatt ohne Export.

## Formate

### Webseite (HTML)

Eine einzige `.html`-Datei, in die alle Screenshots und die Hows-Schrift eingebettet sind. Sie öffnet sich in jedem Browser, ohne Internetverbindung und ohne Hows. Das ist das empfohlene Format zum Teilen per Mail, Chat oder im Intranet. Nutzt das Windows der Lesenden den dunklen Modus, zeigt die Seite die dunkle Palette deines Stils. Ausdrucke nutzen immer die helle.

### PDF-Dokument

Ein PDF zum Drucken oder Archivieren. Papierformat (<!-- fact:export.papers ui:paperNames -->A4 oder US Letter<!-- /fact -->) und Seitenrand kommen aus den [Einstellungen](settings.md#export). Die Hows-Schrift ist eingebettet, deshalb erscheinen Umlaute, Akzente und Buchstaben wie das polnische ł so wie im Editor. Pfeilzeichen fehlen der Schrift, deshalb schreibt das PDF einen Pfeil als Bindestrich und Größer-als-Zeichen.

### Hows-Datei (.steps)

Das bearbeitbare Original: alle Schritte, Texte, Markierungen und die unmarkierten Screenshots. Exportier dieses Format, wenn du später an einer Anleitung weiterarbeiten willst. Es landet in deinem Anleitungsordner, nicht im Exportordner, und erscheint in der Bibliothek unter <!-- fact:ui.recent -->**Zuletzt geöffnet**<!-- /fact -->. Das Format ist offen und in [Das `.steps`-Dateiformat](file-format.md) beschrieben.

### Markdown und JSON

Die App bietet diese Formate noch nicht an. Das Kommandozeilenwerkzeug `steps-cli` schreibt sie aus jeder `.steps`-Datei:

- **Markdown**: ein Ordner mit `guide.md` und einem Ordner `images`, praktisch für Wikis und Issue-Tracker.
- **JSON**: die Anleitung als Daten, für Skripte und andere Werkzeuge.

Siehe [Kommandozeile](cli.md#eine-anleitung-exportieren).

## Aussehen und Hinweiszeile

HTML- und PDF-Exporte nutzen <!-- fact:ui.brand -->**Stil**<!-- /fact --> und <!-- fact:ui.accentColor -->**Akzentfarbe**<!-- /fact --> aus den [Einstellungen](settings.md#darstellung). Ist deine eigene Akzentfarbe auf der Seite schwer lesbar, dunkelt Hows sie ab oder hellt sie auf, nach derselben Regel wie in der Oberfläche.

Standardmäßig tragen Exporte eine kurze Zeile „<!-- fact:i18n.export.credit -->Erstellt mit Hows<!-- /fact -->“ in der Sprache der Anleitung. Sie steht am Ende eines HTML-Exports und in der Fußzeile jeder PDF-Seite. Sie enthält keinen Link und keine Daten über dich. Willst du sie weglassen, nimm unter [Einstellungen](settings.md#export) den Haken bei <!-- fact:ui.exportCredit -->**„Erstellt mit Hows“ in Exporten zeigen**<!-- /fact --> heraus. Die `.steps`-Datei enthält diese Zeile nie.

`steps-cli` schreibt dasselbe Aussehen und dieselbe Zeile, auch in Markdown. Es nutzt den Standardstil, außer du gibst `--brand`, `--accent` oder `--no-credit` an. Siehe [Kommandozeile](cli.md#eine-anleitung-exportieren).

## Was ein Export enthält

Exporte aus der App sind als Anleitung für Lesende geschrieben: der Titel, die nummerierten Schritttexte und die Screenshots mit deinen Markierungen. Sie lassen die getrennten technischen Felder weg, etwa Fenstertitel, Klickposition, Zeitstempel und Systemangaben. Ein Schritttext kann trotzdem eine sichtbare Schaltfläche oder ein Feld nennen und den Fenster- oder Seitentitel; siehe [Datenschutz](privacy.md#exporte).

`steps-cli` kann außerdem im **Fehlerberichtsmodus** exportieren, der diese technischen Angaben für Entwicklerinnen und Entwickler ergänzt. Siehe [Kommandozeile](cli.md#anleitungsmodus-und-fehlerberichtsmodus) und [Datenschutz](privacy.md#exporte).

## Ordner und Dateinamen

| Format | Ordner | Standard |
|---|---|---|
| HTML, PDF | Exportordner | Dein Ordner Dokumente |
| `.steps` | Anleitungsordner | <!-- fact:app.guides_subfolder -->`Steps`<!-- /fact --> in deinem Ordner Dokumente |

Der Dateiname folgt dem Muster für Dateinamen unter [Einstellungen](settings.md#erweitert), standardmäßig dem Titel der Anleitung. **Ein Export ersetzt nie eine vorhandene Datei.** Ist der Name belegt, hängt Hows eine Nummer an, zum Beispiel `Anleitung (2).html`, dann `Anleitung (3).html`. Das gilt auch für `.steps`-Dateien. Jedes Speichern legt eine neue Datei neben der früheren an. Nimm `{date}` und `{time}` in das Muster für Dateinamen auf, wenn du Namen ohne Nummer lieber magst.

Kann Hows einen Ordner nicht nutzen, speichert es stattdessen im Standardordner und sagt dir das.

## OneDrive-Warnung

Windows verschiebt den Ordner Dokumente oft nach OneDrive. Synchronisiert OneDrive den Zielordner, warnt dich das Blatt vor dem Export. Die Screenshots würden in der Cloud landen. Mit **Ändern…** wählst du stattdessen einen lokalen Ordner.

---

[Zur Übersicht](README.md) · Weiter: [Einstellungen](settings.md)
