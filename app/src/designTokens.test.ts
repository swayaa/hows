import assert from "node:assert/strict";
import { readdirSync, readFileSync } from "node:fs";
import { describe, it } from "node:test";
import { BRANDS, paletteVariables } from "./brand.ts";

const SRC = new URL("./", import.meta.url);
const STYLES = new URL("./styles/", import.meta.url);
/** The one place that may name colors and fonts directly. */
const TOKENS_FILE = "tokens.css";

const COLOR_FUNCTION = /\b(?:rgba?|hsla?|hwb|lab|lch|oklab|oklch)\(/;
const HEX = /#[0-9a-fA-F]{3,8}\b/;
const NAMED_COLOR =
  /(?<![\w-])(?:white|black|red|green|blue|gray|grey|yellow|orange|purple|pink|silver|navy|teal|maroon)(?![\w-])/;

type Source = { name: string; text: string; css: string };

function read(dir: URL, name: string): string {
  return readFileSync(new URL(name, dir), "utf8");
}

function withoutComments(text: string): string {
  return text.replace(/\/\*[\s\S]*?\*\//g, "").replace(/<!--[\s\S]*?-->/g, "");
}

/** CSS parts of a component: style blocks and style attributes. */
function componentCss(source: string): string {
  const blocks = [...source.matchAll(/<style[^>]*>([\s\S]*?)<\/style>/g)].map((m) => m[1]);
  const attributes = [...source.matchAll(/\bstyle(?::[\w-]+)?="([^"]*)"/g)].map((m) => m[1]);
  return [...blocks, ...attributes].join("\n");
}

function sources(): Source[] {
  const styles = readdirSync(STYLES)
    .filter((name) => name.endsWith(".css") && name !== TOKENS_FILE)
    .map((name) => {
      const text = withoutComments(read(STYLES, name));
      return { name: `styles/${name}`, text, css: text };
    });
  const components = readdirSync(SRC)
    .filter((name) => name.endsWith(".svelte"))
    .map((name) => {
      const text = withoutComments(read(SRC, name));
      return { name, text, css: componentCss(text) };
    });
  const scripts = readdirSync(SRC)
    .filter((name) => name.endsWith(".ts") && !name.endsWith(".test.ts"))
    .map((name) => ({ name, text: read(SRC, name), css: "" }));
  return [...styles, ...components, ...scripts];
}

function colorLiterals(source: Source): string[] {
  const hits: string[] = [];
  for (const pattern of [HEX, COLOR_FUNCTION]) {
    const match = source.text.match(pattern);
    if (match) hits.push(match[0]);
  }
  const named = source.css.match(NAMED_COLOR);
  if (named) hits.push(named[0]);
  return hits;
}

/**
 * Properties whose lengths come from a scale in tokens.css. Allowed as literals:
 * `0` (no unit), `1px` hairlines in spacing (for example the visually hidden
 * clip), and `%` or `em`, which follow the parent box or the text size.
 */
const SCALES: { name: string; property: RegExp; hairline: boolean }[] = [
  { name: "spacing", property: /^(?:margin|padding|gap|row-gap|column-gap|inset|top|right|bottom|left)(?:-[a-z-]+)?$/, hairline: true },
  { name: "font size", property: /^font-size$/, hairline: false },
  { name: "radius", property: /^border(?:-[a-z]+)*-radius$/, hairline: false },
];
const LENGTH = /(?<![\w.])-?(\d*\.?\d+)(px|rem)(?![\w-])/g;

function offScaleLengths(source: Source): string[] {
  const hits: string[] = [];
  for (const [, property, value] of source.css.matchAll(/(?:^|[;{\s])([a-z-]+)\s*:\s*([^;{}]+)/g)) {
    const scale = SCALES.find((candidate) => candidate.property.test(property));
    if (!scale) continue;
    for (const [literal] of value.matchAll(LENGTH)) {
      if (scale.hairline && /^-?1px$/.test(literal)) continue;
      hits.push(`${source.name}: ${scale.name} ${property}: ${literal}`);
    }
  }
  return hits;
}

function definedVariables(): Set<string> {
  const defined = new Set(BRANDS.flatMap((brand) => Object.keys(paletteVariables(brand.light))));
  const files = [read(STYLES, TOKENS_FILE), ...sources().map((source) => source.text)];
  for (const text of files) {
    for (const match of text.matchAll(/(--sl-[\w-]+)\s*:/g)) defined.add(match[1]);
    for (const match of text.matchAll(/style:(--sl-[\w-]+)/g)) defined.add(match[1]);
  }
  return defined;
}

describe("design tokens", () => {
  it("keeps color values out of styles, components and scripts", () => {
    const found = sources()
      .map((source) => [source.name, colorLiterals(source)] as const)
      .filter(([, hits]) => hits.length > 0)
      .map(([name, hits]) => `${name}: ${hits.join(", ")}`);
    assert.deepEqual(found, []);
  });

  it("uses the font only through the token", () => {
    const found = sources()
      .flatMap((source) =>
        [...source.css.matchAll(/font-family\s*:\s*([^;}"]+)/g)].map((m) => [source.name, m[1].trim()]),
      )
      .filter(([, value]) => !/^(?:var\(--sl-font-[\w-]+\)|inherit)$/.test(value))
      .map(([name, value]) => `${name}: ${value}`);
    assert.deepEqual(found, []);
  });

  it("takes spacing, font sizes and radii from the scales in tokens.css", () => {
    assert.deepEqual(sources().flatMap(offScaleLengths), []);
  });

  it("notices a length that is off the scale", () => {
    const css = ".x { padding: 0.6rem 1px; font-size: 13px; border-radius: 0 0 5px 5px; margin: calc(100% - 1rem); gap: 0.5em; }";
    assert.deepEqual(offScaleLengths({ name: "probe", text: css, css }), [
      "probe: spacing padding: 0.6rem",
      "probe: font size font-size: 13px",
      "probe: radius border-radius: 5px",
      "probe: radius border-radius: 5px",
      "probe: spacing margin: 1rem",
    ]);
  });

  it("only uses variables that a palette or tokens.css defines", () => {
    const defined = definedVariables();
    const used = new Set(
      [read(STYLES, TOKENS_FILE), ...sources().map((source) => source.text)].flatMap((text) =>
        [...text.matchAll(/var\((--sl-[\w-]+)/g)].map((m) => m[1]),
      ),
    );
    assert.deepEqual([...used].filter((name) => !defined.has(name)).sort(), []);
  });
});
