// PDF paper sizes and margin (core/export/src/pdf.rs) and the style presets
// (core/export/brands.json).

import { block, readJson, rustConstant, rustEnum } from "./source.mjs";

const pdf = "core/export/src/pdf.rs";
const brands = "core/export/brands.json";

/** Settings code of each `Paper` variant, from `Paper::code`. */
export function paperCodes() {
  const arms = new Map(
    [...block(pdf, "pub fn code(self)").matchAll(/Paper::(\w+) => "([^"]+)"/g)].map((arm) => [arm[1], arm[2]]),
  );
  const { variants, fallback } = rustEnum(pdf, "Paper");
  const code = (variant) => {
    if (!arms.has(variant)) throw new Error(`${pdf}: Paper::${variant} has no code`);
    return arms.get(variant);
  };
  return { codes: variants.map(code), fallback: code(fallback) };
}

export function defaultBrand() {
  return readJson(brands).default;
}

export default function exportFacts() {
  const papers = paperCodes();
  const catalog = readJson(brands);
  return [
    { key: "export.papers", value: papers.codes, source: pdf },
    { key: "export.default_paper", value: papers.fallback, source: pdf },
    { key: "export.default_margin_mm", value: rustConstant("DEFAULT_MARGIN_MM").value, source: pdf },
    { key: "brands.codes", value: catalog.brands.map((brand) => brand.code), source: brands },
    { key: "brands.default", value: catalog.default, source: brands },
  ];
}
