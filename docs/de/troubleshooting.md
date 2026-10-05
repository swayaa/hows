[English](../en/troubleshooting.md) | Deutsch

# Probleme lösen

<!-- fact-exempt: ui.errors.hotkey_invalid ui.errors.hotkey_taken ui.errors.hotkey_conflict ui.errors.invalid_folder ui.errors.path_missing ui.errors.reveal_failed ui.errors.window_failed -->

## Der Computer wurde durch Windows geschützt

Hows ist noch nicht codesigniert. Siehe [Die SmartScreen-Warnung](smartscreen.md).

## Die portable App startet nicht

`hows.exe` im portablen ZIP braucht die Microsoft-Edge-WebView2-Runtime. Windows 11 bringt sie mit. Unter Windows 10 installierst du die WebView2-Runtime von Microsoft oder nimmst einen der Installer, der sie für dich ergänzt. Siehe [Hows installieren](install.md#webview2).

## Tastenkürzel nicht verfügbar

Eine andere App nutzt das Aufnahme- oder Pause-Kürzel schon. Beim Start zeigt Hows dann <!-- fact:ui.hotkeyRegisterError -->**Tastenkürzel nicht verfügbar**<!-- /fact --> mit dem Kürzel, öffnet die Einstellungen und markiert das Kürzel mit <!-- fact:ui.hotkeyTakenAtStartup -->**Eine andere App nutzt dieses Tastenkürzel schon. Klick ins Feld und drück ein neues.**<!-- /fact --> Wähl eine Kombination, die keine andere App nutzt. Siehe [Ein Tastenkürzel ändern](hotkeys.md#ein-tastenkürzel-ändern).

Die Meldung <!-- fact:ui.hotkeyConflict -->**Dieses Tastenkürzel ist schon vergeben.**<!-- /fact --> erscheint, wenn du eine Kombination wählst, die Windows, eine andere App oder das zweite Kürzel von Hows schon nutzt. Hows behält dann das bisherige Kürzel.

## Die Aufnahme startet nicht

Die Meldung <!-- fact:ui.errors.input_hook_failed -->**Die Aufnahme ist nicht gestartet, weil Hows keine Maus- und Tastatureingaben empfängt. Starte Hows neu und versuch es noch einmal. Passiert das wieder, prüfe, ob eine Sicherheitssoftware Hows blockiert.**<!-- /fact --> heißt, Windows hat Hows nicht auf Klicks und Tasten hören lassen. Hows bleibt in der Bibliothek oder im Editor und nimmt nichts auf. Starte Hows neu. Kommt die Meldung wieder, bitte die Person, die die Sicherheitssoftware auf deinem PC verwaltet, Hows zuzulassen.

## Es wurden keine Schritte aufgenommen

Die Aufnahme endete ohne einen einzigen Schritt. Prüfe, ob die Aufnahme pausiert war, und starte eine neue. Klicks auf die eigenen Fenster von Hows werden nie zu Schritten.

## Ein Schritt hat einen allgemeinen Text oder keinen Screenshot

- Allgemeine Texte wie *Klicke in „…“* kommen von Apps, die Windows wenig über ihre Schaltflächen verraten. Schreib den Text im Editor um.
- Tipp-Schritte haben nie einen Screenshot, damit deine Eingaben privat bleiben. Der Schritt davor zeigt meistens das Feld.
- Konnte kein Screenshot aufgenommen werden, bleibt der Schritt trotzdem erhalten und zeigt <!-- fact:ui.noScreenshot -->**Dieser Schritt hat keinen Screenshot.**<!-- /fact -->

## Eine Anleitung lässt sich nicht öffnen

| Meldung | Bedeutung |
|---|---|
| <!-- fact:ui.errors.not_found -->**Datei nicht gefunden**<!-- /fact --> | Die Datei wurde verschoben, umbenannt oder gelöscht. Die Bibliothek markiert solche Einträge mit <!-- fact:ui.missing -->**Datei fehlt**<!-- /fact -->. |
| <!-- fact:ui.errors.not_steps -->**Das ist keine Hows-Datei (.steps).**<!-- /fact --> | Die Datei ist keine `.steps`-Datei. |
| <!-- fact:ui.errors.corrupt -->**Diese Anleitung lässt sich nicht öffnen. Die Datei ist vielleicht beschädigt.**<!-- /fact --> | Die Datei ist unvollständig oder kein gültiges `.steps`-Archiv. |
| <!-- fact:ui.errors.newer_version -->**Diese Anleitung stammt aus einer neueren Hows-Version. Aktualisiere Hows, um sie zu öffnen.**<!-- /fact --> | Die Datei nutzt ein neueres [Dateiformat](file-format.md), als dein Hows kennt. |

## Speichern oder Exportieren klappt nicht

| Meldung | Was du tun kannst |
|---|---|
| <!-- fact:ui.errors.save_failed -->**Die Anleitung ließ sich nicht speichern. Prüfe, ob du in den Anleitungsordner schreiben darfst.**<!-- /fact --> | Wähl unter [Einstellungen](settings.md#ordner) einen anderen Anleitungsordner, oder schaff Platz auf dem Laufwerk. |
| <!-- fact:ui.errors.export_failed -->**Der Export ließ sich nicht schreiben. Prüfe, ob du in den Exportordner schreiben darfst und ob die Datei in einer anderen App offen ist.**<!-- /fact --> | Schließe den vorigen Export, zum Beispiel in einem PDF-Programm, und exportiere noch einmal. |
| <!-- fact:ui.errors.folder_unavailable -->**Hows kann den Ordner nicht anlegen oder öffnen. Wähl in den Einstellungen einen anderen Ordner.**<!-- /fact --> | Der Ordner liegt auf einem Laufwerk, das fehlt oder schreibgeschützt ist. Wähl einen lokalen Ordner. |
| <!-- fact:ui.errors.settings_not_saved -->**Deine Einstellungen ließen sich nicht speichern. Prüfe, ob du in deinen Benutzerordner schreiben darfst.**<!-- /fact --> | Prüfe die Rechte des Ordners <!-- fact:app.identifier -->`how.hows`<!-- /fact --> unter `%APPDATA%` und ändere die Einstellung dann noch einmal. |

Die Meldung <!-- fact:ui.errors.internal -->**In Hows ist etwas schiefgelaufen. Starte Hows neu und versuch es noch einmal.**<!-- /fact --> heißt, Hows ist auf einen Fehler gestoßen, mit dem es nicht rechnet. Passiert es wieder, melde es wie unten beschrieben.

## Ein Fenster zum Auswählen geht nicht auf

Die Meldung <!-- fact:ui.errors.dialog_failed -->**Das Fenster zum Auswählen einer Datei oder eines Ordners ging nicht auf. Versuch es noch einmal.**<!-- /fact --> erscheint, wenn Windows das Fenster hinter **Öffnen…**, **Durchsuchen…** oder **Ändern…** nicht zeigen konnte. Klick noch einmal auf die Schaltfläche. Geht das Fenster immer noch nicht auf, starte Hows neu.

## Exporte landen am falschen Ort

- Der Hinweis <!-- fact:ui.pathInvalid -->**Diesen Ordner kann Hows nicht nutzen. Es speichert stattdessen im Standardordner.**<!-- /fact --> heißt, der gewählte Ordner fehlt oder ist nicht beschreibbar. Wähl im Exportblatt oder unter [Einstellungen](settings.md#export) einen anderen.
- Die Warnung <!-- fact:ui.exportSyncedWarning -->**OneDrive synchronisiert diesen Ordner. Die Screenshots landen damit in der Cloud.**<!-- /fact --> erscheint vor dem Export in einen synchronisierten Ordner. Wähl mit <!-- fact:ui.exportChangeFolder -->**Ändern…**<!-- /fact --> einen lokalen Ordner, wenn die Anleitung deinen PC nicht verlassen soll.
- Ein Dateiname mit Nummer am Ende, etwa `Anleitung (2).html`, heißt: Eine Datei mit diesem Namen gab es schon. Hows ersetzt sie nie. Siehe [Ordner und Dateinamen](export.md#ordner-und-dateinamen).

## Noch nichts zu exportieren

Die Meldung <!-- fact:ui.noSessionExport -->**Noch nichts zu exportieren. Nimm zuerst eine Anleitung auf.**<!-- /fact --> oder <!-- fact:ui.errors.no_guide -->**Es ist keine Anleitung geöffnet. Nimm eine auf oder öffne eine.**<!-- /fact --> erscheint, wenn der Editor keine Anleitung enthält. Nimm eine auf oder öffne eine `.steps`-Datei.

## Kommst du nicht weiter?

Eröffne ein Issue mit der Vorlage für Fehler und beachte die Tipps unter [Mitmachen](../../CONTRIBUTING.de.md#melden). Sicherheitsprobleme meldest du stattdessen nach der [Sicherheitsrichtlinie](../../SECURITY.de.md).

---

[Zur Übersicht](README.md)
