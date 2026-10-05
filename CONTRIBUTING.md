English | [Deutsch](CONTRIBUTING.de.md)

# Contributing to Hows

Thanks for helping. Bug reports, incorrect step text, and pull requests are all welcome.

## Reporting

- **Bug:** use the bug template. Include the Windows version, display scaling, and what you expected versus what the step list shows.
- **Incorrect step text:** use the step-text template. The most useful attachment is the step's `element` block from `guide.json` (open the `.steps` file as a ZIP) or a bug-report JSON export: `steps-cli export guide.steps --format json --mode bug-report -o report.json`. Remove anything confidential first.
- **Security:** do not open a public issue; see [SECURITY.md](SECURITY.md).

## Toolchain

- Rust stable, 1.88 or newer (`rustup component add clippy rustfmt`)
- Node.js 22
- Windows 10/11 for anything touching capture; Linux works for the core, the CLI, and the frontend
- For the desktop app: the [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/)

## Layout

| Path | What |
|---|---|
| `core/capture` | Input hooks, screenshots, UI Automation. Windows code lives in `src/windows/` behind `cfg(windows)`; other platforms get a stub. |
| `core/session` | Turns raw input events into steps: grouping, double-click and scroll merging, template texts (`texts.rs`). Pure logic, fully tested with fake events. |
| `core/store` | The `.steps` format (`model.rs`), see [the format description](docs/en/file-format.md). |
| `core/export` | HTML, PDF, Markdown, and JSON exports; annotation rendering. |
| `core/cli` | `steps-cli`: `export`, `demo`, `from-script`. |
| `app/src-tauri` | Tauri backend: commands, recorder thread, settings, tray, and keyboard shortcuts. |
| `app/src` | Svelte 5 frontend. UI texts live in `locales/`, one file per language. |
| `docs/en`, `docs/de` | User documentation, one page per topic in each language. |
| `scripts/docs` | Checks for the documentation: language pairs, links, spelling, and values that must match the code. |

## Checks

CI runs these for every pull request, with the Rust and app checks running on Ubuntu **and** Windows. Run them locally before pushing:

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

Clippy runs with the pedantic lint group. The Windows runner is the source of truth for `cfg(windows)` code; if you work on Linux, CI will catch what you cannot compile locally.

### End-to-end smoke test (Windows)

`scripts/smoke/run-sandbox-smoke.ps1` installs a release build into a disposable [Windows Sandbox](https://learn.microsoft.com/windows/security/application-security/application-isolation/windows-sandbox/) without network access, records a short session with synthetic mouse and keyboard input, and checks that a prepared settings file loads, shortcuts register, steps are detected, typed text is not stored, a title is suggested, recent guides are listed, and all exports work. Your own desktop receives no input.

```powershell
cd app; npm run tauri build; cd ..
pwsh scripts/smoke/run-sandbox-smoke.ps1            # or -Bundle <unzipped CI installer artifact>
```

It needs Windows Pro/Enterprise with the optional feature enabled once (admin, then reboot): `Enable-WindowsOptionalFeature -Online -FeatureName Containers-DisposableClientVM -All`. The results, log, and screenshots are written to `%LOCALAPPDATA%\hows-smoke\<timestamp>\results`; `-KeepOpen` leaves the sandbox running. `in-sandbox.ps1` refuses to run outside the sandbox.

The sandbox has no WebView2 runtime, so the first run downloads Microsoft's offline installer (about 200 MB) once, checks its signature, and caches it in `%LOCALAPPDATA%\hows-smoke\cache`. The test talks to the app over the WebView2 DevTools port. The sandbox user is elevated, and WebView2 ignores `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` in elevated processes since runtime 150, so the port is enabled through the `HKLM` WebView2 policy inside the sandbox.

## Rules the code keeps

- **Never record typed text.** Only modifier shortcuts become `key_combo`; everything else is a `text_input` step without content. Pull requests that weaken this will not be merged.
- **No network.** No telemetry, no update pings, no remote fonts.
- **Step texts are deterministic templates** from one JSON file per language in `core/i18n/locales`. No AI, no randomness. A new language adds that file and an entry in `LOCALES` (`core/i18n/src/lib.rs`), plus a UI file in `app/src/locales`; tests check that every language has every key with the same placeholders.
- **The `.steps` format stays readable.** New fields are optional; breaking changes bump `schema_version` and update `docs/en/file-format.md` and `docs/de/file-format.md` (a test parses their examples).
- Switches over enums and unions are exhaustive: in TypeScript with a `never` check in `default`.
- Parts of the code and its comments are in German. New comments may be in English.

## Documentation

The documentation is in English and German, and both versions stay in sync:

- **Where pages live.** User guide pages are `docs/en/<page>.md` and `docs/de/<page>.md` with the same file name. Other documents use `NAME.md` for English and `NAME.de.md` for German, for example `README.md` and `README.de.md`.
- **Language line.** Every page starts with a line that links to its translation: `English | [Deutsch](…)` on English pages, `[English](…) | Deutsch` on German pages.
- **Change both.** A pull request that changes a page updates its translation too. English pages are the reference; German pages use the informal “du”.
- **Facts from the code.** Name settings, buttons, and messages exactly as the app shows them (`app/src/locales/`), with defaults and ranges from `app/src-tauri/src/settings.rs`. Do not describe features before they ship.
- **Mark values.** Wrap each default, range, shortcut, label, and message you quote in a fact marker, for example `<!-- fact:settings.typing_pause_ms seconds -->3<!-- /fact -->`. The key names a value in the code: `settings.<field>`, `limits.<field>.min`, `.max`, or `.step`, `ui.<path>` for app texts, `i18n.<key>` for step and export texts, and the `errors`, `export`, `brands`, `store`, `cli`, `app`, and `build` families in `scripts/docs/facts/`. Optional steps after the key turn the value into the form the page shows: `seconds` (milliseconds as seconds), `keys` (key names of the page language), `bool` (on or off), `upper` (capital letters), and `ui:<name>` (the label the app shows for a code, for example `ui:paperNames`).
- **Marker rules.** Put the marker outside bold text, and do not start a line with it, because Markdown then renders the whole line as raw HTML. Each translation marks the same facts as the English page. Every error code and every settings range must be marked in each language; list error codes the app never shows in a `<!-- fact-exempt: code code -->` comment.
- **Checks.** `npm run check` in `scripts/docs` fails when a translation or the language line is missing, when a relative link or anchor is broken, when a wiki page is not linked from anywhere, when cspell finds an unknown word, or when a marked value differs from the code. Add correct project terms to `scripts/docs/project-words.txt`. External links are not fetched.

## Pull requests

- Branch from `main`, keep each PR to one topic, and describe what changed and how you tested it.
- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.
- Add an entry under *Unreleased* in [CHANGELOG.md](CHANGELOG.md) and [CHANGELOG.de.md](CHANGELOG.de.md) for user-visible changes.
- UI changes: add a screenshot to the PR.

By contributing, you agree that your work is released under the [MIT License](LICENSE).
