/** Brand presets from `core/export/brands.json`, the only source of UI colors. */

import catalog from "../../core/export/brands.json" with { type: "json" };

export type Brand = (typeof catalog.brands)[number];
export type Palette = Brand["light"];
export type Skin = "light" | "dark";

export const BRANDS: readonly Brand[] = catalog.brands;
export const DEFAULT_BRAND: string = catalog.default;

/** Minimum contrast of the accent against the background (WCAG AA for text). */
const ACCENT_CONTRAST = 4.5;

export function findBrand(code: string): Brand {
  return (
    BRANDS.find((brand) => brand.code === code) ??
    BRANDS.find((brand) => brand.code === DEFAULT_BRAND) ??
    BRANDS[0]
  );
}

function channels(hex: string): [number, number, number] {
  const value = Number.parseInt(hex.slice(1), 16);
  return [(value >> 16) & 255, (value >> 8) & 255, value & 255];
}

function toHex([r, g, b]: [number, number, number]): string {
  return `#${[r, g, b].map((c) => Math.round(c).toString(16).padStart(2, "0")).join("").toUpperCase()}`;
}

/** `a` moved toward `b` by `amount` (0..1), in sRGB. */
export function mixHex(a: string, b: string, amount: number): string {
  const [from, to] = [channels(a), channels(b)];
  return toHex([0, 1, 2].map((i) => from[i] + (to[i] - from[i]) * amount) as [number, number, number]);
}

function luminance(hex: string): number {
  const [r, g, b] = channels(hex).map((c) => {
    const s = c / 255;
    return s <= 0.039_28 ? s / 12.92 : ((s + 0.055) / 1.055) ** 2.4;
  });
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

export function contrast(a: string, b: string): number {
  const [x, y] = [luminance(a), luminance(b)];
  return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
}

/** Own accent adjusted toward the text color until it reads on the background. */
export function readableAccent(accent: string, palette: Palette): string {
  for (let step = 0; step <= 10; step += 1) {
    const candidate = mixHex(accent, palette.ink, step / 10);
    if (contrast(candidate, palette.bg) >= ACCENT_CONTRAST) return candidate;
  }
  return palette.ink;
}

/** Palette for a preset and theme, with an optional own accent (`#RRGGBB`, empty = preset). */
export function brandPalette(code: string, skin: Skin, ownAccent: string): Palette {
  const palette = findBrand(code)[skin];
  if (!/^#[0-9A-Fa-f]{6}$/.test(ownAccent)) return palette;
  const accent = readableAccent(ownAccent, palette);
  const onAccent = [palette.on_accent, palette.ink, palette.bg].reduce((best, candidate) =>
    contrast(candidate, accent) > contrast(best, accent) ? candidate : best,
  );
  return { ...palette, accent, on_accent: onAccent };
}

/** CSS custom properties `--sl-<token>` for `palette`. */
export function paletteVariables(palette: Palette): Record<string, string> {
  return Object.fromEntries(
    Object.entries(palette).map(([token, value]) => [`--sl-${token.replaceAll("_", "-")}`, value]),
  );
}

/** Sets the palette on `root`, usually `document.documentElement`. */
export function applyPalette(root: HTMLElement, palette: Palette): void {
  for (const [name, value] of Object.entries(paletteVariables(palette))) {
    root.style.setProperty(name, value);
  }
}
