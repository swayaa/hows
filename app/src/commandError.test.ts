import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { describe, it } from "node:test";
import { asCommandError, errorMessage } from "./commandError.ts";
import { UI_LOCALES, uiLocale } from "./locales/index.ts";

const SOURCES = new URL("./", import.meta.url);
const BACKEND = new URL("../src-tauri/src/", import.meta.url);
const en = uiLocale("en").messages;
const de = uiLocale("de").messages;

/** Variants of `ErrorCode` in `error.rs`, as serde writes them (`SaveFailed` becomes `save_failed`). */
function backendCodes(): string[] {
  const source = readFileSync(new URL("error.rs", BACKEND), "utf8");
  const body = /pub enum ErrorCode \{([\s\S]*?)\n\}/.exec(source)?.[1] ?? "";
  return [...body.matchAll(/^\s{4}([A-Z]\w*),$/gm)].map(([, name]) =>
    name.replace(/[A-Z]/g, (letter, index) => (index ? "_" : "") + letter.toLowerCase()),
  );
}

describe("backend errors", () => {
  it("has a text for every backend code in every language, and no stale ones", () => {
    const codes = backendCodes();
    assert.ok(codes.length >= 10, `found only ${codes.length} codes in error.rs`);
    for (const locale of UI_LOCALES) {
      assert.deepEqual(Object.keys(locale.messages.errors).sort(), [...codes].sort(), locale.code);
    }
  });

  it("shares the codes of the core error texts the CLI prints", () => {
    const codes = backendCodes();
    const core = new URL("../../core/i18n/locales/", import.meta.url);
    for (const file of readdirSync(core).filter((name) => name.endsWith(".json"))) {
      const texts = JSON.parse(readFileSync(new URL(file, core), "utf8")) as Record<string, string>;
      const shared = Object.keys(texts)
        .filter((key) => key.startsWith("error."))
        .map((key) => key.slice("error.".length));
      assert.ok(shared.length >= 6, `${file} has only ${shared.length} error texts`);
      for (const code of shared) assert.ok(codes.includes(code), `${file}: error.${code} is no backend code`);
    }
  });

  it("translates the code and never shows the backend's own words", () => {
    const failure = { code: "save_failed", detail: "Zugriff verweigert (os error 5)" };
    assert.equal(errorMessage(failure, en), en.errors.save_failed);
    assert.equal(errorMessage(failure, de), de.errors.save_failed);
    assert.equal(errorMessage({ code: "not_found", detail: "" }, de), "Datei nicht gefunden");
    for (const raw of ["ungültiger Zustandsübergang: Idle → Paused", "toString", { code: "boom" }, null]) {
      assert.equal(errorMessage(raw, en), en.errors.internal, String(raw));
    }
  });

  it("keeps the technical detail and the default folder apart from the text", () => {
    assert.deepEqual(asCommandError({ code: "invalid_folder", detail: "Z:\\gone", path: "C:\\Docs" }), {
      code: "invalid_folder",
      detail: "Z:\\gone",
      path: "C:\\Docs",
    });
    assert.deepEqual(asCommandError("E/A-Fehler"), { code: "internal", detail: "E/A-Fehler" });
  });

  it("routes every caught error through the translation", () => {
    const offenders = readdirSync(SOURCES)
      .filter((name) => /\.(svelte|ts)$/.test(name) && !name.endsWith(".test.ts") && name !== "commandError.ts")
      .filter((name) => /String\((error|err|e|reason)\)/.test(readFileSync(new URL(name, SOURCES), "utf8")));
    assert.deepEqual(offenders, []);
  });

  it("answers every command failure with a code, not a sentence", () => {
    const offenders = readdirSync(BACKEND)
      .filter((name) => name.endsWith(".rs"))
      .filter((name) => {
        const source = readFileSync(new URL(name, BACKEND), "utf8");
        return /Result<[^;{]*, String>/.test(source) || /map_err\(\|\w+\| \w+\.to_string\(\)\)/.test(source);
      });
    assert.deepEqual(offenders, []);
  });
});
