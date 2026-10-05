import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { describe, it } from "node:test";
import catalog from "../../core/export/brands.json" with { type: "json" };
import { BRANDS, brandPalette, contrast, DEFAULT_BRAND, findBrand, mixHex, paletteVariables } from "./brand.ts";

/** Same vectors as `derivation_matches_the_app` in `core/export/src/brand.rs`. */
const PARITY = [
  ["sage", "light", "#FFE14D", "#5F5D2E", "#FFFFFF"],
  ["sage", "dark", "#1E40AF", "#6F86C6", "#0E1512"],
  ["ink", "light", "#AA3366", "#AA3366", "#FFFFFF"],
  ["ember", "dark", "#10B981", "#10B981", "#1A1512"],
] as const;

describe("brand presets", () => {
  it("falls back to the default preset for unknown codes", () => {
    assert.equal(findBrand("neon").code, DEFAULT_BRAND);
    assert.equal(findBrand(BRANDS[1].code).code, BRANDS[1].code);
  });

  it("turns palette tokens into --sl- variables", () => {
    const vars = paletteVariables(findBrand(DEFAULT_BRAND).light);
    assert.equal(vars["--sl-accent"], findBrand(DEFAULT_BRAND).light.accent);
    assert.ok("--sl-on-accent" in vars && "--sl-film-ink" in vars);
  });

  it("mixes colors in sRGB", () => {
    assert.equal(mixHex("#000000", "#FFFFFF", 0.5), "#808080");
    assert.equal(mixHex("#123456", "#FFFFFF", 0), "#123456");
  });

  it("keeps the preset accent when no own accent is set or it is invalid", () => {
    for (const own of ["", "teal", "#12345"]) {
      assert.deepEqual(brandPalette(DEFAULT_BRAND, "light", own), findBrand(DEFAULT_BRAND).light);
    }
  });

  it("makes any own accent readable in both themes", () => {
    for (const brand of BRANDS) {
      for (const skin of ["light", "dark"] as const) {
        for (const own of ["#FFE14D", "#1E40AF", "#10B981", "#000000", "#FFFFFF", "#E11D48"]) {
          const palette = brandPalette(brand.code, skin, own);
          const label = `${brand.code} ${skin} ${own}`;
          assert.ok(contrast(palette.accent, palette.bg) >= 4.5, `${label}: accent on bg`);
          assert.ok(contrast(palette.on_accent, palette.accent) >= 3, `${label}: text on accent`);
        }
      }
    }
  });

  it("derives the same accent as the exports", () => {
    for (const [code, skin, own, accent, onAccent] of PARITY) {
      const palette = brandPalette(code, skin, own);
      assert.equal(palette.accent, accent, `${code} ${skin} ${own}`);
      assert.equal(palette.on_accent, onAccent, `${code} ${skin} ${own}`);
    }
  });

  it("uses the brand font of the exports", () => {
    const tokens = readFileSync(new URL("./styles/tokens.css", import.meta.url), "utf8");
    const stack = /--sl-font-ui:\s*([^;]+);/.exec(tokens)?.[1] ?? "";
    const families = stack.split(",").map((name) => name.trim().replace(/^"|"$/g, ""));
    assert.deepEqual(families, [`${catalog.font.family} Variable`, ...catalog.font.fallback]);
  });
});
