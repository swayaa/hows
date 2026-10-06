[English](../en/smartscreen.md) | Deutsch

# Die SmartScreen-Warnung

Hows-Builds sind noch nicht codesigniert. Microsoft Defender SmartScreen warnt vor Programmen, deren Herausgeber es nicht prüfen kann. Wenn du den Installer oder `hows.exe` aus dem portablen ZIP zum ersten Mal startest, zeigt Windows deshalb vielleicht **Der Computer wurde durch Windows geschützt**.

## Was die Warnung bedeutet

SmartScreen weiß nicht, wer die Datei veröffentlicht hat. Das heißt nicht, dass ein Virus gefunden wurde. Starte trotzdem nur Dateien, denen du vertraust:

- Lade Hows nur von der Seite [Releases](https://github.com/swayaa/hows/releases) dieses Repositorys herunter. Siehe [Hows installieren](install.md#herunterladen).
- Wenn du einer fertigen Datei nicht vertrauen willst, [baue Hows selbst](../../README.de.md#bauen).

## Hows trotzdem starten

1. Klick im blauen SmartScreen-Fenster auf **Weitere Informationen**.
2. Prüfe, ob der App-Name zu der Datei passt, die du heruntergeladen hast.
3. Klick auf **Trotzdem ausführen**.

## Wenn die Datei blockiert ist

Windows markiert Dateien, die aus dem Internet kommen. Ist eine heruntergeladene ZIP-Datei oder eine Datei darin blockiert:

1. Klick mit der rechten Maustaste auf die heruntergeladene ZIP-Datei und wähle **Eigenschaften**.
2. Setz auf dem Tab **Allgemein** den Haken bei **Zulassen** und klick auf **OK**.
3. Entpacke die ZIP-Datei noch einmal.

Manche Organisationen verwalten SmartScreen zentral. Siehst du keine Schaltfläche **Trotzdem ausführen**, bitte deine IT-Abteilung, Hows freizugeben.

## Signatur und Ruf

Codesignierung ist geplant. Signierte Builds tragen den geprüften Namen des Herausgebers. Ob SmartScreen dann noch warnt, hängt vom Ruf ab, den Herausgeber und Datei bei SmartScreen aufgebaut haben. Auch ein neues signiertes Release kann deshalb eine Zeit lang die Warnung zeigen.

---

[Zur Übersicht](README.md) · Weiter: [Eine Anleitung aufnehmen](recording.md)
