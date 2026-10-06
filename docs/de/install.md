[English](../en/install.md) | Deutsch

# Hows installieren

Hows läuft unter Windows 10 und Windows 11.

## Herunterladen

Hows 0.1.0 ist ein frühes Alpha. Die Builds sind unsigniert, deshalb warnt Windows SmartScreen beim ersten Start. [Die SmartScreen-Warnung](smartscreen.md) erklärt, warum, und wie du weitermachst.

Lade Hows von der Seite [Releases](https://github.com/swayaa/hows/releases) herunter. Für die meisten ist das Setup-Programm die richtige Datei. Spätere Versionen stehen auf derselben Seite.

## Die passende Datei wählen

Das Release 0.1.0 enthält diese Dateien:

| Datei | Geeignet für |
|---|---|
| `Hows_0.1.0_x64-setup.exe` | Die meisten. Installiert Hows für dich und meldet `.steps`-Dateien an, damit ein Doppelklick sie in Hows öffnet. |
| `Hows_0.1.0_x64_en-US.msi` | Installation auf vielen PCs mit Softwareverteilung. Meldet `.steps`-Dateien ebenfalls an. |
| `Hows_0.1.0_x64-portable.zip` | Nutzung ohne Installation. Es enthält `hows.exe`, `LICENSE` und `THIRD-PARTY-NOTICES.md`. Es meldet `.steps`-Dateien nicht an. |

`wix-UIExtension-wix3141rtm.zip` ist die Quelle und die Lizenz der WiX-Dialoge, die zum MSI gehören. Sie ist kein Installer. Die meisten brauchen sie nicht. Gibst du das MSI weiter, bleibt das Archiv dabei.

Ein erfolgreicher Lauf des Workflows **Release** legt außerdem das Artefakt `windows-installers` ab. Diese ZIP-Datei ist ein CI-Build. Sie ist nicht der Download für den normalen Gebrauch.

## WebView2

Hows zeichnet sein Fenster mit Microsoft Edge WebView2. Windows 11 bringt es schon mit. Fehlt es, laden beide Installer es während der Einrichtung von Microsoft herunter. Das ist die einzige Netzwerkverbindung der Installer; nach der Installation baut die laufende App keine Netzwerkverbindungen auf.

`hows.exe` im portablen ZIP installiert WebView2 nicht. Fehlt es auf dem PC, installiere zuerst die WebView2-Runtime von Microsoft.

## Erster Start

- Hows öffnet seine Bibliothek und legt ein Symbol im Infobereich der Taskleiste ab.
- Die Oberfläche folgt deiner Windows-Anzeigesprache, wenn Hows sie kann (Englisch, Deutsch, Französisch, Spanisch, Italienisch, Portugiesisch oder Niederländisch), sonst Englisch. Du kannst sie unter [Einstellungen](settings.md#darstellung) ändern.
- Nutzt eine andere App schon eines der Aufnahme-Tastenkürzel, öffnet Hows die Einstellungen und markiert das Kürzel. Siehe [Tastenkürzel](hotkeys.md#ein-tastenkürzel-ändern).

## Aktualisieren

Hows sucht nicht nach Updates. Nach der Installation baut die laufende App keine Netzwerkverbindungen auf. Zum Aktualisieren lädst du den neuen Build herunter und startest seinen Installer. Deine Anleitungen und Einstellungen liegen außerhalb des Programmordners und bleiben erhalten.

## Deinstallieren

Öffne in Windows **Einstellungen**, dann **Apps**, dann **Installierte Apps**, und deinstalliere **Hows**. Beim portablen ZIP löschst du den entpackten Ordner.

Deine Anleitungen und Exporte werden nicht gelöscht; sie bleiben in den Ordnern, die du gewählt hast (standardmäßig in deinem Ordner Dokumente). Seine Einstellungen speichert Hows in einer Datei `settings.json` in seinem Ordner unter `%APPDATA%`. Lösche diesen Ordner, wenn du auch sie entfernen willst.

## Selbst bauen

Du kannst Hows auch aus dem Quellcode bauen. Der [Projektüberblick](../../README.de.md#bauen) nennt die Voraussetzungen und Befehle.

---

[Zur Übersicht](README.md) · Weiter: [Die SmartScreen-Warnung](smartscreen.md)
