English | [Deutsch](../de/cli.md)

# Command line

`steps-cli` exports existing guides and builds new ones from a JSON script. It needs no desktop and no screen capture, so it runs on Windows and Linux, including in CI. That makes it useful for documentation pipelines and AI agents.

There is no prebuilt `steps-cli` binary yet. Run it from a clone of the repository with Rust installed (see the [project overview](../../README.md#build)):

```bash
cargo run --manifest-path core/Cargo.toml -p steps-cli -- <command> ...
```

For brevity, the examples below use `steps-cli`.

## Language

`steps-cli` writes its help, messages, and the demo guide in the display language of your operating system. If Hows doesn't offer that language, it uses English. To choose a language, add `--lang` to any command:

```bash
steps-cli demo -o ./out --lang fr
```

`--lang` takes `en`, `de`, `fr`, `es`, `it`, `pt`, `nl`, or `system` for the display language. `steps-cli help` lists every option in the chosen language.

## Export a guide

```bash
steps-cli export <input.steps> --format html|pdf|markdown|json [--mode sop|bug-report] -o <path>
  [--brand <code>] [--accent <#rrggbb>] [--no-credit]
```

| Format | Output | `-o` is |
|---|---|---|
| `html` | One self-contained `.html` file | a file |
| `pdf` | One `.pdf` file, <!-- fact:export.default_paper ui:paperNames -->A4<!-- /fact --> | a file |
| `markdown` (or `md`) | `guide.md` plus an `images` folder | a folder |
| `json` | One `.json` file | a file |

`steps-cli` never replaces an existing file or folder. If `-o` is taken, it writes to the next free name, such as `guide (2).html` or `md-out (2)`, and prints the path it used. Every command follows this rule, including `demo` and `from-script`.

Without `--format`, `steps-cli` writes <!-- fact:cli.default_format upper -->HTML<!-- /fact -->. PDF exports from the command line always use <!-- fact:export.default_paper ui:paperNames -->A4<!-- /fact --> with the default margin of <!-- fact:export.default_margin_mm -->18<!-- /fact --> mm; the paper size and margin settings of the app do not apply here.

The command line does not read the app settings for the look either. These options set it:

| Option | Default | What it does |
|---|---|---|
| `--brand` | <!-- fact:brands.default -->`sage`<!-- /fact --> | Style of HTML and PDF: <!-- fact:brands.codes -->`sage`, `ink`, or `ember`<!-- /fact -->. |
| `--accent` | The accent of the style | Your own accent color as `#rrggbb`, adjusted for readability as in the app. |
| `--no-credit` | Off | Leaves out the “<!-- fact:i18n.export.credit -->Created with Hows<!-- /fact -->” line in HTML, PDF, and Markdown. |

In the app, the styles are called <!-- fact:brands.codes ui:brandNames -->Sage, Ink, and Ember<!-- /fact -->.

Example:

```bash
steps-cli export guide.steps --format pdf -o guide.pdf
```

## Guide mode and bug-report mode

In the default mode, <!-- fact:cli.default_mode -->`sop`<!-- /fact -->, `steps-cli` writes a guide for readers: title, description, step texts, and screenshots with marks. This is what the app exports.

`--mode bug-report` also writes the creation time, the operating system and, if the file contains one, its version, and, for each step, the app, window title, element, time, click position, and monitor details. Use it to attach a recording to a bug report. Check the output first. Window titles can contain private names. See [Privacy](privacy.md#exports).

```bash
steps-cli export guide.steps --format json --mode bug-report -o report.json
```

## Build a guide from a script

```bash
steps-cli from-script <script.json> -o <out.steps> [--html [<out.html>]]
```

`--html` also writes an HTML export. Without a path, it lands next to the `.steps` file with the same name.

A script looks like this ([full example](../examples/agent-steps.en.json)):

```json
{
  "title": "Map a network drive",
  "description": "Optional text under the title.",
  "language": "en",
  "steps": [
    { "text": "Open File Explorer (Win+E)." },
    { "title": "Menu", "text": "Choose \"Map network drive\".", "image": "./menu.png" }
  ]
}
```

| Field | Required | Meaning |
|---|---|---|
| `title` | yes | Guide title. |
| `description` | no | Text under the title. |
| `language` | no | Language code of the guide, for example `en`. Defaults to the language of `steps-cli`, see [Language](#language). |
| `steps[].text` | yes | Step text. |
| `steps[].title` | no | Short prefix, shown as `Title: text`. |
| `steps[].image` | no | Path to a PNG screenshot, relative to the script file. Steps without an image get an empty placeholder. |

The script needs at least one step.

## Demo

```bash
steps-cli demo -o ./out
```

Writes a demo guide as `.steps` plus every export format into `./out`, without any recording. This is a handy way to see what the exports look like.

---

[Documentation overview](README.md) · Next: [The `.steps` file format](file-format.md)
