[English](../en/install.md) | Deutsch

# Hows installieren

Hows läuft unter Windows 10 und Windows 11.

## Herunterladen

Es gibt noch kein signiertes Release. Die Windows-Builds kommen aus dem Workflow **Release** dieses Repositorys:

1. Öffne den Tab **Actions** und wähle den Workflow **Release**.
2. Öffne einen erfolgreichen Lauf und lade unter **Artifacts** das Artefakt `windows-installers` herunter. Es ist eine ZIP-Datei.
3. Entpacke die ZIP-Datei.

Sobald eine Version veröffentlicht ist, findest du dieselben Dateien auch in ihrem Eintrag auf der Seite **Releases**.

Beim ersten Start warnt Windows SmartScreen vor den unsignierten Dateien. [Die SmartScreen-Warnung](smartscreen.md) erklärt, warum, und wie du weitermachst.

## Die passende Datei wählen

Das Artefakt enthält den Installer, das MSI, das portable ZIP und `wix-UIExtension-wix3141rtm.zip`. Du brauchst eine der ersten drei Dateien. Gibst du das MSI weiter, bleibt das WiX-Archiv dabei. Es ist die Quelle der WiX-Dialoge und ihrer Lizenz, kein Weg, Hows zu installieren.

| Datei | Geeignet für |
|---|---|
| Installer mit der Endung `-setup.exe` | Die meisten. Installiert Hows für dich und meldet `.steps`-Dateien an, damit ein Doppelklick sie in Hows öffnet. |
| Paket mit der Endung `.msi` | Verteilung in Firmen mit Softwareverteilung. Meldet `.steps`-Dateien ebenfalls an. |
| `Hows_*_x64-portable.zip` | Portable Nutzung ohne Installation. Es enthält `hows.exe`, `LICENSE` und `THIRD-PARTY-NOTICES.md`. Es meldet `.steps`-Dateien nicht an. |

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
