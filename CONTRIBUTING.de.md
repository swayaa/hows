[English](CONTRIBUTING.md) | Deutsch

# Bei Hows mitmachen

Danke, dass du hilfst. Fehlerberichte, falsche Schritttexte und Pull Requests sind willkommen.

## Melden

- **Fehler:** Nutze die Vorlage für Fehler. Gib die Windows-Version und die Anzeigeskalierung an und beschreibe, was du erwartet hast und was die Schrittliste zeigt.
- **Schritttext klingt falsch:** Nutze die Vorlage für Schritttexte. Am meisten hilft der Block `element` des Schritts aus `guide.json` (öffne die `.steps`-Datei als ZIP) oder ein JSON-Export im Fehlerberichtsmodus: `steps-cli export guide.steps --format json --mode bug-report -o report.json`. Entferne vorher alles Vertrauliche.
- **Sicherheit:** Eröffne kein öffentliches Issue, siehe [SECURITY.de.md](SECURITY.de.md).

## Werkzeuge

- Rust stable, 1.88 oder neuer (`rustup component add clippy rustfmt`)
- Node.js 22
- Windows 10/11 für alles, was die Aufnahme berührt; für den Kern, die Kommandozeile und das Frontend reicht Linux
- Für die Desktop-App: die [Tauri-Voraussetzungen](https://v2.tauri.app/start/prerequisites/)

## Aufbau

| Pfad | Inhalt |
|---|---|
| `core/capture` | Eingabe-Hooks, Screenshots, UI Automation. Der Windows-Code liegt in `src/windows/` hinter `cfg(windows)`; andere Plattformen bekommen einen Platzhalter. |
| `core/session` | Macht aus rohen Eingabe-Ereignissen Schritte: Gruppieren, Zusammenführen von Doppelklicks und Scrollen, Textvorlagen (`texts.rs`). Reine Logik, vollständig mit simulierten Ereignissen getestet. |
| `core/store` | Das `.steps`-Format (`model.rs`), siehe [die Beschreibung des Formats](docs/de/file-format.md). |
| `core/export` | Exporte als HTML, PDF, Markdown und JSON, Zeichnen der Markierungen. |
| `core/cli` | `steps-cli`: `export`, `demo`, `from-script`. |
| `app/src-tauri` | Tauri-Backend: Befehle, Aufnahme-Thread, Einstellungen, Infobereich und Tastenkürzel. |
| `app/src` | Svelte-5-Frontend. Die Texte der Oberfläche liegen in `locales/`, eine Datei pro Sprache. |
| `docs/en`, `docs/de` | Benutzerdokumentation, eine Seite pro Thema in jeder Sprache. |
| `scripts/docs` | Prüfungen der Dokumentation: Sprachpaare, Links, Rechtschreibung und Werte, die zum Code passen müssen. |

## Prüfungen

CI führt sie für jeden Pull Request aus, die Rust- und App-Prüfungen unter Ubuntu **und** Windows. Lass sie vor dem Pushen lokal laufen:

```bash
# core
cargo fmt --manifest-path core/Cargo.toml --all -- --check
cargo clippy --manifest-path core/Cargo.toml --workspace --all-targets -- -D warnings
cargo test --manifest-path core/Cargo.toml --workspace

# app frontend
cd app
npm ci
npm run check    # svelte-check, warnings fail
npm test
npm run build
cd ..

# app backend
cargo fmt --manifest-path app/src-tauri/Cargo.toml --all -- --check
cargo clippy --manifest-path app/src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path app/src-tauri/Cargo.toml

# documentation
cd scripts/docs
npm ci
npm run check    # language pairs, relative links, spelling, facts
cd ../..
```

Clippy läuft mit der Lint-Gruppe `pedantic`. Maßgeblich für `cfg(windows)`-Code ist der Windows-Runner; arbeitest du unter Linux, fängt CI ab, was du lokal nicht kompilieren kannst.

### End-to-End-Smoke-Test (Windows)

`scripts/smoke/run-sandbox-smoke.ps1` installiert einen Release-Build in eine [Windows-Sandbox](https://learn.microsoft.com/windows/security/application-security/application-isolation/windows-sandbox/) ohne Netzwerk, die beim Schließen alles verwirft, nimmt mit simulierten Maus- und Tastatureingaben eine kurze Sitzung auf und prüft, dass eine vorbereitete Einstellungsdatei geladen wird, die Registrierung der Tastenkürzel, die Schritterkennung, dass getippter Text nicht gespeichert wird, den Titelvorschlag, die zuletzt geöffneten Anleitungen und alle Exporte. Dein eigener Desktop bekommt keine Eingaben.

```powershell
cd app; npm run tauri build; cd ..
pwsh scripts/smoke/run-sandbox-smoke.ps1            # or -Bundle <unzipped CI installer artifact>
```

Er braucht Windows Pro oder Enterprise mit einmal aktiviertem optionalem Feature (als Administrator, danach Neustart): `Enable-WindowsOptionalFeature -Online -FeatureName Containers-DisposableClientVM -All`. Ergebnisse, Log und Screenshots landen in `%LOCALAPPDATA%\hows-smoke\<timestamp>\results`; mit `-KeepOpen` bleibt die Sandbox offen. `in-sandbox.ps1` verweigert den Start außerhalb der Sandbox.

Die Sandbox hat keine WebView2-Runtime. Der erste Lauf lädt deshalb einmal Microsofts Offline-Installer (etwa 200 MB), prüft seine Signatur und legt ihn in `%LOCALAPPDATA%\hows-smoke\cache` ab. Der Test spricht über den DevTools-Port von WebView2 mit der App. Der Sandbox-Nutzer hat erhöhte Rechte, und WebView2 ignoriert seit Runtime 150 `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` in Prozessen mit erhöhten Rechten; deshalb schaltet eine WebView2-Richtlinie unter `HKLM` in der Sandbox den Port frei.

## Regeln, die der Code einhält

- **Nie getippten Text aufnehmen.** Nur Kürzel mit Modifikatortaste werden `key_combo`; alles andere wird ein Schritt `text_input` ohne Inhalt. Pull Requests, die das aufweichen, werden nicht übernommen.
- **Kein Netzwerk.** Keine Telemetrie, keine Update-Abfragen, keine Schriften aus dem Netz.
- **Schritttexte sind feste Vorlagen** aus einer JSON-Datei pro Sprache in `core/i18n/locales`. Keine KI, kein Zufall. Eine neue Sprache bekommt diese Datei, einen Eintrag in `LOCALES` (`core/i18n/src/lib.rs`) und eine Datei für die Oberfläche in `app/src/locales`; Tests prüfen, dass jede Sprache jeden Schlüssel mit denselben Platzhaltern hat.
- **Das `.steps`-Format bleibt lesbar.** Neue Felder sind optional; inkompatible Änderungen erhöhen `schema_version` und aktualisieren `docs/en/file-format.md` und `docs/de/file-format.md` (ein Test liest ihre Beispiele).
- Switches über Enums und Unions sind vollständig: in TypeScript mit einer `never`-Prüfung im `default`.
- Teile des Codes und seiner Kommentare sind auf Deutsch. Neue Kommentare dürfen auf Englisch sein.

## Dokumentation

Die Dokumentation gibt es auf Englisch und Deutsch, und beide Fassungen bleiben gleich auf:

- **Wo Seiten liegen.** Seiten des Benutzerhandbuchs heißen `docs/en/<page>.md` und `docs/de/<page>.md` mit demselben Dateinamen. Andere Dokumente nutzen `NAME.md` für Englisch und `NAME.de.md` für Deutsch, zum Beispiel `README.md` und `README.de.md`.
- **Sprachzeile.** Jede Seite beginnt mit einer Zeile, die ihre Übersetzung verlinkt: `English | [Deutsch](…)` auf englischen Seiten, `[English](…) | Deutsch` auf deutschen.
- **Beide ändern.** Ein Pull Request, der eine Seite ändert, aktualisiert auch ihre Übersetzung. Die englischen Seiten sind die Referenz; die deutschen Seiten nutzen das „du“.
- **Fakten aus dem Code.** Nenn Einstellungen, Schaltflächen und Meldungen genau so, wie die App sie zeigt (`app/src/locales/`), mit Standards und Bereichen aus `app/src-tauri/src/settings.rs`. Beschreibe keine Funktionen, bevor sie erscheinen.
- **Werte markieren.** Markiere jeden Standard, jeden Bereich, jedes Tastenkürzel, jede Beschriftung und jede Meldung, die du zitierst, mit einer Faktenmarkierung, zum Beispiel `<!-- fact:settings.typing_pause_ms seconds -->3<!-- /fact -->`. Der Schlüssel nennt einen Wert im Code: `settings.<field>`, `limits.<field>.min`, `.max` oder `.step`, `ui.<path>` für Texte der App, `i18n.<key>` für Schritt- und Exporttexte sowie die Familien `errors`, `export`, `brands`, `store`, `cli`, `app` und `build` in `scripts/docs/facts/`. Optionale Schritte nach dem Schlüssel bringen den Wert in die Form, die die Seite zeigt: `seconds` (Millisekunden als Sekunden), `keys` (Tastennamen der Seitensprache), `bool` (an oder aus), `upper` (Großbuchstaben) und `ui:<name>` (die Beschriftung, die die App für einen Code zeigt, zum Beispiel `ui:paperNames`).
- **Regeln für Markierungen.** Setz die Markierung außerhalb von Fettdruck und beginne keine Zeile mit ihr, sonst zeigt Markdown die ganze Zeile als rohes HTML. Jede Übersetzung markiert dieselben Fakten wie die englische Seite. Jeder Fehlercode und jeder Bereich einer Einstellung muss in jeder Sprache markiert sein; Fehlercodes, die die App nie zeigt, stehen in einem Kommentar `<!-- fact-exempt: code code -->`.
- **Prüfungen.** `npm run check` in `scripts/docs` schlägt fehl, wenn eine Übersetzung oder die Sprachzeile fehlt, wenn ein relativer Link oder Anker nicht auflöst, wenn keine andere Seite auf eine Seite des Handbuchs verlinkt, wenn cspell ein unbekanntes Wort findet oder wenn ein markierter Wert vom Code abweicht. Richtige Fachbegriffe des Projekts gehören in `scripts/docs/project-words.txt`. Externe Links werden nicht abgerufen.

## Pull Requests

- Zweige von `main` ab, halte jeden PR bei einem Thema und beschreibe, was sich geändert hat und wie du es getestet hast.
- Commit-Nachrichten folgen [Conventional Commits](https://www.conventionalcommits.org/de/): `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.
- Trag sichtbare Änderungen unter *Unreleased* in [CHANGELOG.md](CHANGELOG.md) und [CHANGELOG.de.md](CHANGELOG.de.md) ein.
- Änderungen an der Oberfläche: Hänge dem PR einen Screenshot an.

Mit deinem Beitrag stimmst du zu, dass deine Arbeit unter der [MIT-Lizenz](LICENSE) veröffentlicht wird.
