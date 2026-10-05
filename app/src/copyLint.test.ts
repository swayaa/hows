import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { describe, it } from "node:test";
import { UI_LOCALES } from "./locales/index.ts";

const COMPONENTS = new URL("./", import.meta.url);
const DASH = /[\u2013\u2014]/;
const TEXT_ARROW = /[\u2190-\u21FF]|->|<-/;

const RULES: { name: string; pattern: RegExp }[] = [
  { name: "dash", pattern: DASH },
  { name: "text arrow", pattern: TEXT_ARROW },
  {
    name: "internal term",
    pattern: /\b(pill|primary|soft ledger|sop|byo|chrome|filmstrip|fallback|prefs)\b/i,
  },
];

/** Exceptions to the rules. Only removals allowed. */
const KNOWN_SLOP = new Set<string>([]);

function collectStrings(value: unknown, path: string, out: Map<string, string>): void {
  if (typeof value === "string") {
    out.set(path, value);
  } else if (value && typeof value === "object") {
    for (const [key, child] of Object.entries(value)) collectStrings(child, `${path}.${key}`, out);
  }
}

function violations(): Map<string, string[]> {
  const strings = new Map<string, string>();
  for (const locale of UI_LOCALES) collectStrings(locale.messages, locale.code, strings);
  const found = new Map<string, string[]>();
  for (const [path, text] of strings) {
    const broken = RULES.filter((rule) => rule.pattern.test(text)).map((rule) => rule.name);
    if (broken.length > 0) found.set(path, broken);
  }
  return found;
}

/** Symbol glyphs standing in for icons (arrows, math, technical, shapes, dingbats). */
const GLYPH = /[\u2190-\u2BFF\uFE0F]/;

/** Components whose markup still breaks a rule. Only removals allowed. */
const KNOWN_MARKUP_SLOP = new Set<string>([]);

/** Visible part of a component: without script, style and HTML comments. */
function markup(source: string): string {
  return source
    .replace(/<script[\s\S]*?<\/script>/g, "")
    .replace(/<style[\s\S]*?<\/style>/g, "")
    .replace(/<!--[\s\S]*?-->/g, "");
}

function markupViolations(): string[] {
  return readdirSync(COMPONENTS)
    .filter((name) => name.endsWith(".svelte"))
    .filter((name) => {
      const visible = markup(readFileSync(new URL(name, COMPONENTS), "utf8"));
      return DASH.test(visible) || TEXT_ARROW.test(visible) || GLYPH.test(visible);
    });
}

describe("UI copy lint", () => {
  it("keeps dashes, text arrows and icon glyphs out of component markup", () => {
    const fresh = markupViolations().filter((name) => !KNOWN_MARKUP_SLOP.has(name));
    assert.deepEqual(fresh, []);
  });

  it("lists only components that still break the markup rule", () => {
    const found = new Set(markupViolations());
    assert.deepEqual([...KNOWN_MARKUP_SLOP].filter((name) => !found.has(name)), []);
  });

  it("has no dashes, text arrows or internal terms", () => {
    const fresh = [...violations()]
      .filter(([path]) => !KNOWN_SLOP.has(path))
      .map(([path, rules]) => `${path}: ${rules.join(", ")}`);
    assert.deepEqual(fresh, []);
  });

  it("lists only offenders that still break a rule", () => {
    const found = violations();
    const fixed = [...KNOWN_SLOP].filter((path) => !found.has(path));
    assert.deepEqual(fixed, []);
  });
});
