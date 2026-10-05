[English](../en/privacy.md) | Deutsch

# Datenschutz

Hows braucht kein Konto und sendet keine Telemetrie. Nach der Installation lädt die laufende App nichts hoch und baut keine Netzwerkverbindungen auf. Fehlt Microsofts WebView2-Runtime, laden die NSIS- und MSI-Installer sie während der Einrichtung von Microsoft herunter (siehe [Hows installieren](install.md#webview2)). Das portable ZIP tut das nicht. Hows schickt deine Anleitungen nicht an einen Hows-Cloud-Dienst. Speicherst oder exportierst du in einen Ordner, den OneDrive oder ein anderes Sync-Werkzeug synchronisiert, kann dieses Werkzeug die Datei vom PC kopieren; das Exportblatt warnt, wenn der Exportordner synchronisiert wird.

## Deine Verantwortung beim Aufnehmen und Teilen

Die Aufnahme startet über das Tastenkürzel, die Schaltfläche oder den Infobereich. Hows hat keinen Einwilligungsschritt in der App für andere Personen, und Windows zeigt für diese Hooks keinen Dialog zur Bildschirmaufnahme-Berechtigung. Nimmst du den Bildschirm, den Arbeitsplatz, einen Anruf oder einen Chat einer anderen Person auf oder teilst du das, liegt die Verantwortung bei dir. Dasselbe gilt, wenn du einen Export teilst, der andere Personen oder ihre Daten zeigt.

## Was Hows aufnimmt

| Wird aufgenommen | Wird nicht aufgenommen |
|---|---|
| Klicks, Doppelklicks und Rechtsklicks mit der Position auf dem Bildschirm | Getippte Zeichen als Tastatureingabe |
| Ein Screenshot des Bildschirms bei jedem Klick-, Kürzel- und Scroll-Schritt | Screenshots von Tipp-Schritten |
| Kürzel mit `Strg`, `Alt` oder `Win`, zum Beispiel `Strg+S` | Einzelne Tasten, Kombinationen mit `Umschalt` und Zeichen mit `AltGr` wie `@`, die nur als Tippen zählen |
| Dass du getippt hast, und in welchem Feld | Die eigenen Fenster von Hows, auch Tasten, die du drückst, während ein Fenster von Hows vorne ist |
| Name und Art des angeklickten Elements, Fenstertitel, App-Name | Alles, während die Aufnahme pausiert ist |
| Der Name des Betriebssystems, aber nicht die Windows-Version | |

Hows speichert die Zeichen, die du tippst, nicht als Tastatureingabe. Ein Tipp-Schritt bekommt keinen Screenshot. Der Screenshot eines späteren Schritts kann getippten Text trotzdem zeigen, etwa in einem Feld, das noch offen ist.

## Was auf deinem PC bleibt

- **`.steps`-Dateien** in deinem Anleitungsordner enthalten deine Schritte, meist jeweils mit dem Original-Screenshot, und die Angaben, die Windows dazu geliefert hat: den Fenstertitel, den App-Namen und das angeklickte Element. Fenstertitel können Dokument- oder Kundennamen enthalten.
- **Exporte** landen in deinem Exportordner.
- **Einstellungen** liegen in einer Datei <!-- fact:app.settings_file -->`settings.json`<!-- /fact --> im Ordner <!-- fact:app.identifier -->`how.hows`<!-- /fact --> unter `%APPDATA%`, einschließlich der Liste der zuletzt geöffneten Anleitungen.

Nichts verlässt deinen PC über Hows, außer du kopierst oder teilst selbst eine Datei. Ein synchronisierter Ordner kann eine Datei, die du dort ablegst, trotzdem hochladen.

## Lizenz von Hows, nicht vom Aufgenommenen

Die [MIT-Lizenz](../../LICENSE) gilt für Hows selbst. Sie gilt nicht für den Inhalt deiner Screenshots oder Schritttexte. Screenshots des ganzen Bildschirms sowie Fenster- oder Seitentitel können Software, Dokumente, Chats oder Webseiten anderer Personen zeigen. Eine Anleitung zu veröffentlichen kann Rechte brauchen, die du unabhängig von der Hows-Lizenz hast.

## Exporte

Exporte aus der App enthalten nur die Anleitung: den Titel, die Schritttexte und die Screenshots mit deinen Markierungen. Sie lassen die getrennten technischen Felder weg: Elementangaben, App-Name, Fenstertitel, Klickposition, Zeitstempel, Bildschirm und Betriebssystem.

Die Schritttexte, die Hows schreibt, nennen, was du angeklickt hast und wo, zum Beispiel *<!-- fact:i18n.step.click.target_in_context -->Klicke auf „{target}“ in „{context}“<!-- /fact -->*. Ein Schritttext kann also den sichtbaren Namen einer Schaltfläche oder eines Felds enthalten und den Fenstertitel, im Browser also den Seitentitel. Fenster- und Seitentitel können Dokument-, Kunden- oder Mailnamen enthalten. Lies vor dem Teilen die Schritttexte genauso wie die Screenshots und ändere einen Text bei Bedarf im Editor.

HTML- und PDF-Exporte tragen außerdem die Zeile „<!-- fact:i18n.export.credit -->Erstellt mit Hows<!-- /fact -->“, solange du sie in den [Einstellungen](settings.md#export) nicht ausschaltest. Die Zeile nennt nur die App. Sie hat keinen Link, kein Tracking und keine Daten über dich oder deinen PC.

Der Fehlerberichtsmodus von `steps-cli` ergänzt Fenstertitel, Elementangaben, Klickpositionen, Zeitstempel, Angaben zum Bildschirm und das Betriebssystem. Die Windows-Version ergänzt er nur, wenn die `.steps`-Datei eine enthält. Aufnahmen aus der App enthalten sie vorerst nicht, ein Fehlerbericht nennt die Windows-Version also nicht immer. Nutze den Fehlerberichtsmodus nur, wenn die Empfänger diese Angaben brauchen. Siehe [Kommandozeile](cli.md#anleitungsmodus-und-fehlerberichtsmodus).

## Bevor du teilst

- Screenshots zeigen alles, was auf dem Bildschirm war, auch Benachrichtigungen, andere Fenster und getippten Text, und können deshalb private Daten enthalten. Prüfe jeden Schritt.
- Nutze das Werkzeug <!-- fact:ui.tools.blur -->**Unschärfe**<!-- /fact --> für Namen, Adressen, Nummern und alles andere, was privat ist. Unscharfe Bereiche sind in jedem Export verpixelt; der ursprüngliche Screenshot in der `.steps`-Datei bleibt unverändert.
- Eine `.steps`-Datei enthält die originalen, unmarkierten Screenshots und die Angaben, die Windows geliefert hat, auch die Fenstertitel. Teile lieber einen HTML- oder PDF-Export, wenn die Empfänger die Anleitung nicht bearbeiten müssen.
- Synchronisiert OneDrive deinen Exportordner, warnt dich das Exportblatt vor dem Export. Siehe [Exportieren und teilen](export.md#onedrive-warnung).

## Ein Problem melden

Sollte Hows jemals getippten Text aufnehmen, Dateien außerhalb der gewählten Ordner schreiben (außer in den Standardordnern, wenn deine nicht nutzbar sind) oder eine Netzwerkverbindung öffnen, melde das bitte vertraulich, wie in der [Sicherheitsrichtlinie](../../SECURITY.de.md) beschrieben.

---

[Zur Übersicht](README.md) · Weiter: [Kommandozeile](cli.md)
