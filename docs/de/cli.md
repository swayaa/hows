[English](../en/cli.md) | Deutsch

# Kommandozeile

`steps-cli` exportiert vorhandene Anleitungen und baut neue aus einem JSON-Skript. Es braucht keinen Desktop und keine Bildschirmaufnahme und läuft deshalb unter Windows, Linux und in CI. Das macht es nützlich für Dokumentations-Pipelines und KI-Agenten.

Ein fertiges `steps-cli`-Programm gibt es noch nicht. Du startest es aus einem Klon des Repositorys mit installiertem Rust (siehe [Projektüberblick](../../README.de.md#bauen)):

```bash
cargo run --manifest-path core/Cargo.toml -p steps-cli -- <command> ...
```

Die Beispiele unten schreiben kurz `steps-cli`.

## Sprache

`steps-cli` schreibt Hilfe, Meldungen und die Demo-Anleitung in der Anzeigesprache deines Betriebssystems. Bietet Hows diese Sprache nicht an, nimmt es Englisch. Eine andere Sprache wählst du mit `--lang` bei jedem Befehl:

```bash
steps-cli demo -o ./out --lang fr
```

`--lang` nimmt `en`, `de`, `fr`, `es`, `it`, `pt`, `nl` oder `system` für die Anzeigesprache. `steps-cli help` zeigt alle Optionen in der gewählten Sprache.

## Eine Anleitung exportieren

```bash
steps-cli export <input.steps> --format html|pdf|markdown|json [--mode sop|bug-report] -o <path>
  [--brand <code>] [--accent <#rrggbb>] [--no-credit]
```

| Format | Ergebnis | `-o` ist |
|---|---|---|
| `html` | Eine eigenständige `.html`-Datei | eine Datei |
| `pdf` | Eine `.pdf`-Datei, <!-- fact:export.default_paper ui:paperNames -->A4<!-- /fact --> | eine Datei |
| `markdown` (oder `md`) | `guide.md` und ein Ordner `images` | ein Ordner |
| `json` | Eine `.json`-Datei | eine Datei |

`steps-cli` ersetzt nie eine vorhandene Datei oder einen vorhandenen Ordner. Ist `-o` belegt, schreibt es unter den nächsten freien Namen, etwa `guide (2).html` oder `md-out (2)`, und gibt den genutzten Pfad aus. Das gilt für jeden Befehl, auch für `demo` und `from-script`.

Ohne `--format` schreibt `steps-cli` <!-- fact:cli.default_format upper -->HTML<!-- /fact -->. PDF-Exporte von der Kommandozeile nutzen immer <!-- fact:export.default_paper ui:paperNames -->A4<!-- /fact --> mit dem Standardrand von <!-- fact:export.default_margin_mm -->18<!-- /fact --> mm; Papierformat und Seitenrand aus den Einstellungen der App gelten hier nicht.

Auch das Aussehen liest die Kommandozeile nicht aus den Einstellungen der App. Diese Optionen legen es fest:

| Option | Standard | Was sie tut |
|---|---|---|
| `--brand` | <!-- fact:brands.default -->`sage`<!-- /fact --> | Stil von HTML und PDF: <!-- fact:brands.codes -->`sage`, `ink` oder `ember`<!-- /fact -->. |
| `--accent` | Der Akzent des Stils | Deine eigene Akzentfarbe als `#rrggbb`, lesbar gemacht wie in der App. |
| `--no-credit` | Aus | Lässt die Zeile „<!-- fact:i18n.export.credit -->Erstellt mit Hows<!-- /fact -->“ in HTML, PDF und Markdown weg. |

In der App heißen die Stile <!-- fact:brands.codes ui:brandNames -->Salbei, Tinte und Glut<!-- /fact -->.

Beispiel:

```bash
steps-cli export guide.steps --format pdf -o guide.pdf
```

## Anleitungsmodus und Fehlerberichtsmodus

Im Standardmodus <!-- fact:cli.default_mode -->`sop`<!-- /fact --> schreibt `steps-cli` eine Anleitung für Lesende: Titel, Beschreibung, Schritttexte und Screenshots mit Markierungen. So exportiert auch die App.

`--mode bug-report` schreibt zusätzlich die Erstellungszeit, das Betriebssystem, seine Version, wenn die Datei eine enthält, und pro Schritt App, Fenstertitel, Element, Zeit, Klickposition und Angaben zum Bildschirm. Damit hängst du eine Aufnahme an einen Fehlerbericht. Prüfe das Ergebnis vorher. Fenstertitel können private Namen enthalten. Siehe [Datenschutz](privacy.md#exporte).

```bash
steps-cli export guide.steps --format json --mode bug-report -o report.json
```

## Eine Anleitung aus einem Skript bauen

```bash
steps-cli from-script <script.json> -o <out.steps> [--html [<out.html>]]
```

`--html` schreibt zusätzlich einen HTML-Export. Ohne Pfad landet er mit demselben Namen neben der `.steps`-Datei.

Ein Skript sieht so aus ([vollständiges Beispiel](../examples/agent-steps.sample.json)):

```json
{
  "title": "Netzlaufwerk verbinden",
  "description": "Optionaler Text unter dem Titel.",
  "language": "de",
  "steps": [
    { "text": "Öffne den Datei-Explorer (Win+E)." },
    { "title": "Menü", "text": "Wähle „Netzlaufwerk verbinden“.", "image": "./menu.png" }
  ]
}
```

| Feld | Pflicht | Bedeutung |
|---|---|---|
| `title` | ja | Titel der Anleitung. |
| `description` | nein | Text unter dem Titel. |
| `language` | nein | Sprachcode der Anleitung, zum Beispiel `en`. Standard ist die Sprache von `steps-cli`, siehe [Sprache](#sprache). |
| `steps[].text` | ja | Schritttext. |
| `steps[].title` | nein | Kurzes Präfix, angezeigt als `Titel: Text`. |
| `steps[].image` | nein | Pfad zu einem PNG-Screenshot, relativ zur Datei des Skripts. Schritte ohne Bild bekommen einen leeren Platzhalter. |

Das Skript braucht mindestens einen Schritt.

## Demo

```bash
steps-cli demo -o ./out
```

Schreibt eine Demo-Anleitung als `.steps` und in jedem Exportformat nach `./out`, ganz ohne Aufnahme. Praktisch, um zu sehen, wie die Exporte aussehen.

---

[Zur Übersicht](README.md) · Weiter: [Das `.steps`-Dateiformat](file-format.md)
