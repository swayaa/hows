import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { describe, it } from "node:test";
import { UI_LOCALES, uiLocale } from "./locales/index.ts";
import { fillTemplate } from "./template.ts";

const CORE_LOCALES = new URL("../../core/i18n/locales/", import.meta.url);

function coreCatalog(code: string): Record<string, string> {
  return JSON.parse(readFileSync(new URL(`${code}.json`, CORE_LOCALES), "utf8"));
}

function leaves(value: unknown, path = "", out = new Map<string, string>()): Map<string, string> {
  if (typeof value === "string") {
    out.set(path, value);
  } else if (value && typeof value === "object") {
    for (const [key, child] of Object.entries(value)) leaves(child, path ? `${path}.${key}` : key, out);
  }
  return out;
}

function placeholders(text: string): string[] {
  return [...text.matchAll(/\{(\w+)\}/g)].map((match) => match[1]).sort();
}

/**
 * `: ; ! ?` before a space or the end and « » inside need U+00A0. The brand font
 * has no U+202F, so a narrow space would come from a fallback font in another width.
 */
function frenchSpacingErrors(text: string): string[] {
  const errors: string[] = [];
  for (const [, before, mark] of text.matchAll(/(.)([:;!?])(?=\s|$)/gu)) {
    if (before !== "\u00a0") errors.push(before + mark);
  }
  for (const [pair] of text.matchAll(/«.|.»/gu)) {
    if (!pair.includes("\u00a0")) errors.push(pair);
  }
  return errors;
}

const english = leaves(UI_LOCALES[0].messages);

describe("UI locale registry", () => {
  it("starts with English and has unique codes", () => {
    assert.equal(UI_LOCALES[0].code, "en");
    const codes = UI_LOCALES.map((locale) => locale.code);
    assert.deepEqual(codes, [...new Set(codes)]);
  });

  it("offers exactly the languages of core/i18n", () => {
    const core = readdirSync(CORE_LOCALES)
      .filter((name) => name.endsWith(".json"))
      .map((name) => name.slice(0, -".json".length));
    assert.deepEqual(UI_LOCALES.map((locale) => locale.code).sort(), core.sort());
  });

  for (const locale of UI_LOCALES) {
    it(`${locale.code} has every English key with the same placeholders`, () => {
      const texts = leaves(locale.messages);
      assert.deepEqual([...texts.keys()].sort(), [...english.keys()].sort());
      for (const [path, text] of texts) {
        assert.notEqual(text.trim(), "", `${locale.code}.${path} is empty`);
        assert.deepEqual(placeholders(text), placeholders(english.get(path) ?? ""), path);
      }
    });

    it(`${locale.code} agrees with core/i18n on name and key names`, () => {
      const core = coreCatalog(locale.code);
      assert.equal(locale.name, core["locale.name"]);
      const { keyNames } = locale.messages;
      assert.deepEqual(
        [keyNames.ctrl, keyNames.shift, keyNames.alt, keyNames.win],
        [core["key.ctrl"], core["key.shift"], core["key.alt"], core["key.win"]],
      );
    });

    it(`${locale.code} names the export credit exactly as exports print it`, () => {
      const credit = coreCatalog(locale.code)["export.credit"];
      assert.ok(credit, "core/i18n has export.credit");
      assert.ok(locale.messages.exportCredit.includes(credit), locale.messages.exportCredit);
    });
  }

  it("French puts a no-break space before : ; ! ? and inside « »", () => {
    const fr = UI_LOCALES.find((locale) => locale.code === "fr");
    assert.ok(fr);
    const texts = [...leaves(fr.messages), ...Object.entries(coreCatalog("fr"))];
    for (const [path, text] of texts) {
      assert.deepEqual(frenchSpacingErrors(text), [], `${path}: ${JSON.stringify(text)}`);
    }
  });

  it("French step headings keep a visible space before the colon", () => {
    const fr = uiLocale("fr").messages;
    assert.equal(fillTemplate(fr.stepHeading, { n: 1, text: "Clique sur OK" }), "Étape 1\u00a0: Clique sur OK");
  });

  it("French spacing check notices every rule", () => {
    assert.deepEqual(frenchSpacingErrors("Étape 1 : texte"), [" :"]);
    assert.deepEqual(frenchSpacingErrors("Vraiment?"), ["t?"]);
    assert.deepEqual(frenchSpacingErrors("Étape 1\u202f: texte"), ["\u202f:"]);
    assert.deepEqual(frenchSpacingErrors("Oui\u202f!"), ["\u202f!"]);
    assert.deepEqual(frenchSpacingErrors("« texte »"), ["« ", " »"]);
    assert.deepEqual(frenchSpacingErrors("Étape\u00a0: «\u00a0x\u00a0» oui\u00a0! à 12:30, C:\\"), []);
  });

  it("resolves region tags and falls back to English", () => {
    assert.equal(uiLocale("de-AT").code, "de");
    assert.equal(uiLocale("DE_de").code, "de");
    assert.equal(uiLocale("klingon").code, "en");
    assert.equal(uiLocale("").code, "en");
  });
});
