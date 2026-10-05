// Defaults and limits of the app settings (app/src-tauri/src/settings.rs):
// `settings.<field>` is the default of a field, `limits.<field>.min|max|step`
// its allowed range. Every limit's minimum and maximum must be documented.

import { defaultBrand, paperCodes } from "./export.mjs";
import { block, rustConstant, rustLiteral } from "./source.mjs";
import { marks } from "./store.mjs";

const file = "app/src-tauri/src/settings.rs";

/** Calls in `Settings::default()` that name a value kept elsewhere. */
const calls = {
  "Paper::default().code().to_owned()": () => paperCodes().fallback,
  "brand::default_brand().to_owned()": defaultBrand,
  "marks().color.clone()": () => marks().color,
  "marks().stroke": () => marks().stroke,
  "String::new()": () => "",
};

function resolve(expression) {
  if (Object.hasOwn(calls, expression)) return { value: calls[expression]() };
  const bare = expression.replace(/\.to_owned\(\)$/, "");
  const literal = rustLiteral(bare);
  if (literal !== undefined) return { value: literal };
  if (/^[A-Z][A-Z0-9_]*$/.test(bare)) return { value: rustConstant(bare).value };
  return { error: `cannot read the default "${expression}" in ${file}` };
}

export default function settingsFacts() {
  const facts = [];
  for (const [, field, expression] of block(file, "impl Default for Settings").matchAll(/^\s*(\w+): (.+),$/gm)) {
    facts.push({ key: `settings.${field}`, ...resolve(expression.trim()), source: file });
  }

  const required = [];
  for (const [, field, body] of block(file, "pub const LIMITS: Limits = Limits").matchAll(/(\w+): Limit \{([^}]*)\}/g)) {
    for (const [, bound, expression] of body.matchAll(/(min|max|step): ([^,\s]+)/g)) {
      facts.push({ key: `limits.${field}.${bound}`, value: rustLiteral(expression), source: file });
    }
    required.push(`limits.${field}.min`, `limits.${field}.max`);
  }
  if (required.length === 0) throw new Error(`${file}: no limits found`);
  return { facts, required };
}
