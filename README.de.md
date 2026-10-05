[English](README.md) | Deutsch

# Hows

*How's how? Hows knows.*

Hows nimmt auf, was du anklickst, und macht daraus eine bebilderte Schritt-für-Schritt-Anleitung. Es ist ein moderner Open-Source-Nachfolger der Windows-Schrittaufzeichnung (`psr.exe`), die Microsoft abgekündigt hat.

Hows braucht kein Konto und sendet keine Telemetrie. Nach der Installation baut die laufende App keine Netzwerkverbindungen auf. Die NSIS- und MSI-Installer laden WebView2 von Microsoft, wenn es fehlt. Hows lädt Anleitungen nicht in eine Hows-Cloud hoch; ein Ordner, den OneDrive oder etwas Ähnliches synchronisiert, kann Dateien, die du dort speicherst, trotzdem kopieren. Getippte Zeichen werden nicht als Tastatureingabe gespeichert, nur dass du getippt hast und wo. Anleitungen werden in einem offenen, dokumentierten `.steps`-Format gespeichert. Die App exportiert sie als einzelne HTML-Datei oder als PDF; die Kommandozeile ergänzt Markdown und JSON, entweder als saubere Anleitung oder als Fehlerbericht mit technischen Metadaten.

Zuerst für Windows. Gebaut mit Tauri 2, Rust und Svelte 5. MIT-lizenziert.

**Dokumentation:** [Benutzerhandbuch und Nachschlagen](docs/de/README.md)

## Was es macht

1. Drück <!-- fact:settings.hotkey keys -->`Strg+Umschalt+R`<!-- /fact --> (oder die Schaltfläche, oder das Symbol im Infobereich) und arbeite wie gewohnt.
2. Jeder Klick, Doppelklick, Rechtsklick und jedes Tastenkürzel mit `Strg`, `Alt` oder `Win` wird ein Schritt mit Screenshot und einem lesbaren Satz wie *Klicke auf „Speichern“ in „Editor“*. Scrollen in einem Fenster und einer Richtung bleibt bis zu einer Pause ein Schritt, und der Satz nennt dieses Fenster und wie weit du gescrollt hast, zum Beispiel *Scrolle 3 in „Dokumente“ nach unten*. Tippen bleibt bis zu einer Pause ein Schritt.
3. Beende mit demselben Kürzel. Prüfe die Schritte, korrigiere Texte, sortiere oder lösche Schritte und markiere Screenshots mit Rechteck, Pfeil, Kreis, Stift, Textmarker, Unschärfe und Text.
4. Exportiere: HTML (eine eigenständige Datei), PDF (<!-- fact:export.papers ui:paperNames -->A4 oder US Letter<!-- /fact -->) oder die `.steps`-Datei zum späteren Bearbeiten. HTML und PDF nutzen deinen Stil, deine Akzentfarbe und die eingebettete Hows-Schrift. Eine kleine Zeile „<!-- fact:i18n.export.credit -->Erstellt mit Hows<!-- /fact -->“ steht am Ende jedes HTML-Exports und in der Fußzeile jeder PDF-Seite; du kannst sie in den Einstellungen ausschalten. `steps-cli` schreibt außerdem Markdown mit Bildern und JSON.

Die App und ihre Schritttexte sprechen Englisch, Deutsch, Französisch, Spanisch, Italienisch, Portugiesisch und Niederländisch. Beim ersten Start folgt Hows der Windows-Anzeigesprache; in den Einstellungen kannst du sie jederzeit wechseln.

## Im Vergleich

| | **Hows** | Windows-Schrittaufzeichnung | [OpenSteps](https://github.com/ebanez8/openstep) | [BetterStepsRecorder](https://github.com/Better-World-Solutions/BetterStepsRecorder-Community) | [Folge](https://folge.me) | Scribe / Tango |
|---|---|---|---|---|---|---|
| Lizenz | MIT | proprietär, abgekündigt | MIT | MIT | proprietär | proprietär |
| Läuft | lokal nach der Installation | lokal | lokal | lokal | lokal | Cloud |
| Tastatur | nie Klartext; nur Kürzel, Tippen als „<!-- fact:i18n.step.text_input -->Gib deinen Text ein<!-- /fact -->“ | Tastennamen im Schritttext | Tippen erkannt, Zeichen nicht gespeichert | Metadaten pro Schritt | ja | ja |
| Schritttext | feste Vorlagen aus UI Automation | roher UI-Automation-Text | Klick-Kontext | Klick-Kontext | manuell/automatisch | KI-generiert |
| Dateiformat | `.steps` (ZIP mit `guide.json` und PNGs), [dokumentiert](docs/de/file-format.md) | `.mht` | `session.json` plus PNGs | projektspezifisch | proprietär | Cloud |
| Export | HTML, PDF, Markdown, JSON; Anleitungsmodus und Fehlerberichtsmodus | MHT | Markdown, HTML | RTF, HTML, ODT, Obsidian | PDF, Word, PowerPoint, HTML, Markdown, JSON, SCORM, GIF | PDF, HTML, Links |
| Markierungen | Rechteck, Pfeil, Kreis, Stift, Textmarker, Unschärfe, Text; zerstörungsfrei | keine | Rechteck, Pfeil, Hervorhebung, Nummern | Textbearbeitung | ja | ja |
| Anleitungen aus Skripten | `steps-cli from-script` baut Anleitungen ohne Desktop | nein | nein | nein | nein | API in bezahlten Tarifen |
| Codesignierung | noch nicht | signiert | noch nicht | entfällt | ja | entfällt |

Quellen: [Microsoft zur Abkündigung der Schrittaufzeichnung](https://support.microsoft.com/en-us/windows/apps/steps-recorder-deprecation), die verlinkten Projektseiten, [Scribe-Preise](https://scribehow.com/pricing), [Tango-Preise](https://www.tango.ai/pricing).

## Installieren

Es gibt noch kein signiertes Release. Bis dahin baust du Hows selbst (siehe unten) oder lädst unter *Actions* das Artefakt `windows-installers` aus einem Lauf des Workflows **Release** herunter. Windows SmartScreen warnt vor dem unsignierten Installer.

- [Hows installieren](docs/de/install.md): welche Datei du nimmst, WebView2, Updates und Deinstallation.
- [Die SmartScreen-Warnung](docs/de/smartscreen.md): warum sie erscheint und wie du weitermachst.

## Bauen

Voraussetzungen: Windows 10/11, Rust stable (<!-- fact:build.rust_version -->1.88<!-- /fact --> oder neuer), Node.js 22 und die [Tauri-Voraussetzungen](https://v2.tauri.app/start/prerequisites/) (WebView2 ist in Windows 11 enthalten).

```bash
cd app
npm ci
npm run tauri build
```

| Ergebnis | Pfad |
|---|---|
| NSIS-Installer | `app/src-tauri/target/release/bundle/nsis/*-setup.exe` |
| MSI | `app/src-tauri/target/release/bundle/msi/*.msi` |
| `hows.exe` | `app/src-tauri/target/release/hows.exe` |

`npm run tauri build` schreibt `hows.exe`. Die portable Verteilung ist das ZIP aus `scripts/release/pack-portable.ps1`: `hows.exe`, `LICENSE` und `THIRD-PARTY-NOTICES.md`. Der Release-Workflow veröffentlicht dieses ZIP und nicht `hows.exe` allein.

Für ein Entwicklungsfenster mit Hot Reload: `npm run tauri dev`.

## Kommandozeile

`steps-cli` läuft auf jeder Plattform und braucht keine Bildschirmaufnahme. Es exportiert vorhandene Anleitungen und baut neue aus einem JSON-Skript, praktisch für CI, Dokumentations-Pipelines und KI-Agenten.

```bash
# Demo-Anleitung plus jedes Exportformat
cargo run --manifest-path core/Cargo.toml -p steps-cli -- demo -o ./out

# Eine .steps-Datei exportieren (html | pdf | markdown | json; --mode sop | bug-report)
cargo run --manifest-path core/Cargo.toml -p steps-cli -- \
  export guide.steps --format pdf -o guide.pdf

# Eine Anleitung aus einem Skript bauen; --html ohne Pfad schreibt guide.html daneben
cargo run --manifest-path core/Cargo.toml -p steps-cli -- \
  from-script docs/examples/agent-steps.en.json -o guide.steps --html
```

Alle Befehle und das Format der Skripte: [Kommandozeile](docs/de/cli.md).

## Datenschutz

Nach der Installation baut die laufende App keine Netzwerkverbindungen auf. Fehlt Microsofts WebView2-Runtime (in Windows 11 enthalten), laden die NSIS- und MSI-Installer sie von Microsoft herunter. Das portable ZIP tut das nicht. Hows lädt Anleitungen nicht in eine Hows-Cloud hoch. Die `.steps`-Datei auf deiner Festplatte enthält deine Schritte, meist jeweils mit dem Original-Screenshot, und die Angaben, die Windows geliefert hat: den Fenstertitel, den App-Namen und das angeklickte Element.

Die Aufnahme startet über das Tastenkürzel, die Schaltfläche oder den Infobereich. Hows hat keinen Einwilligungsschritt in der App für andere Personen, und Windows zeigt für diese Hooks keinen Dialog zur Bildschirmaufnahme-Berechtigung. Den Bildschirm, den Arbeitsplatz, einen Anruf oder einen Chat einer anderen Person aufzunehmen oder zu teilen, liegt in deiner Verantwortung.

Exporte aus der App nutzen den Anleitungsmodus: Titel, Schritttexte und Screenshots, ohne die getrennten technischen Felder. Ein Schritttext kann trotzdem eine sichtbare Schaltfläche oder ein Feld nennen und den Fenstertitel, im Browser also den Seitentitel. Der Fehlerberichtsmodus (`steps-cli export --mode bug-report`) ergänzt Fenstertitel, Elementangaben, Klickpositionen, Zeitstempel und das Betriebssystem. Die Windows-Version ergänzt er nur, wenn die Datei eine enthält, und Aufnahmen aus der App enthalten vorerst keine. Screenshots zeigen alles, was auf dem Bildschirm war, und können private Daten enthalten, also prüfe sie und die Schritttexte vor dem Teilen; das Werkzeug Unschärfe verdeckt heikle Bereiche. Die MIT-Lizenz gilt für Hows, nicht für die aufgenommene Oberfläche in diesen Screenshots oder Titeln.

Exporte landen standardmäßig in deinem Ordner Dokumente. Synchronisiert OneDrive diesen Ordner, sagt dir das Exportblatt das vor dem Export und lässt dich einen anderen Ordner wählen. Mehr dazu: [Datenschutz](docs/de/privacy.md).

## Bekannte Grenzen

- Builds sind noch nicht codesigniert, deshalb warnt SmartScreen beim ersten Start.
- Aufnehmen funktioniert nur unter Windows. Unter Linux läuft die App mit einem Platzhalter statt der Aufnahme, die Kommandozeile funktioniert voll; macOS ist nicht getestet.
- Schritte kommen aus UI Automation. Apps, die Windows wenig über ihre Bedienelemente mitteilen (manche Spiele, Remotedesktops, selbst gezeichnete Oberflächen), bekommen allgemeine Schritttexte.
- Die App exportiert HTML, PDF und `.steps`; für Markdown und JSON brauchst du vorerst `steps-cli`.
- Die Browser-Erweiterung für genauere Web-Schritte (`extension/`) ist geplant, aber noch leer.

## Aufbau des Repositorys

```
core/        Rust-Workspace: capture, i18n, session, store, export, cli
app/         Desktop-App mit Tauri 2 und Svelte 5
extension/   Browser-Erweiterung (geplant)
docs/        Benutzerdokumentation (en, de), Dateiformat und Beispiele
scripts/     Smoke-Test und Prüfungen der Dokumentation
```

## Mitmachen

Fehlerberichte, Schritttexte, die falsch klingen, und Pull Requests sind willkommen. Siehe [CONTRIBUTING.de.md](CONTRIBUTING.de.md). Sicherheitsprobleme: [SECURITY.de.md](SECURITY.de.md). Änderungen: [CHANGELOG.de.md](CHANGELOG.de.md).

## Lizenz

[MIT](LICENSE). Die Lizenz gilt für Hows, nicht für den Inhalt von Screenshots oder Schritttexten in einer Anleitung. Fremdkomponenten und ihre Lizenzen: [THIRD-PARTY-NOTICES.de.md](THIRD-PARTY-NOTICES.de.md).
