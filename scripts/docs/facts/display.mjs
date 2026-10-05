// How a code value appears on a page. Display steps follow the fact key,
// separated by spaces, and apply to each item of a list:
//
//   seconds      milliseconds as seconds (3000 -> 3)
//   mib          bytes as MiB (16777216 -> 16)
//   keys         shortcut with the key names of the language (Ctrl -> Strg)
//   bool         the language's word for on or off (config.json `words`)
//   upper        upper case (html -> HTML)
//   ui:<name>    the interface label of the value: `ui.<name>.<value>`,
//                else `ui.<name><Value>` (sage -> Sage or Salbei)
//
// Numbers follow the language's decimal mark, or `.` inside code. Lists are
// written with commas and the language's "or" or "and".

import { config } from "../lib.mjs";

const wordsOf = new Map(config.languages.map((language) => [language.code, language.words]));

const capitalize = (text) => text.charAt(0).toUpperCase() + text.slice(1);

const steps = {
  seconds: (value) => value / 1000,
  mib: (value) => value / 1048576,
  upper: (value) => String(value).toUpperCase(),
  bool: (value, { words }) => words[value ? "on" : "off"],
  keys: (value, { label }) =>
    value
      .split("+")
      .map((part) => label(`ui.keyNames.${part.trim().toLowerCase()}`) ?? part.trim())
      .join("+"),
  ui: (value, { label }, name) => {
    const text = label(`ui.${name}.${value}`) ?? label(`ui.${name}${capitalize(String(value))}`);
    if (text === undefined) throw new Error(`no interface label ui.${name}.${value} or ui.${name}${capitalize(String(value))}`);
    return text;
  },
};

/** The value the page should show, after the display steps. */
export function expectedValue(marker, language, valueOf) {
  const context = { words: wordsOf.get(language), label: (key) => valueOf(key, language) };
  let value = valueOf(marker.key, language);
  if (value === undefined) throw new Error(`${marker.key} has no value in ${language}`);
  for (const step of marker.steps) {
    const [name, argument] = step.split(":");
    if (!Object.hasOwn(steps, name)) throw new Error(`unknown display step "${step}"`);
    const apply = (item) => steps[name](item, context, argument);
    value = Array.isArray(value) ? value.map(apply) : apply(value);
  }
  return value;
}

function decimalMark(language) {
  return new Intl.NumberFormat(language).formatToParts(1.5).find((part) => part.type === "decimal").value;
}

/** Text without surrounding emphasis; `code` is true if it was one code span. */
function unwrap(raw) {
  const text = raw.replace(/\s+/g, " ").trim().replace(/^\*+(.*?)\*+$/, "$1");
  const code = /^`([^`]+)`$/.exec(text);
  return code ? { text: code[1], code: true } : { text, code: false };
}

function parseNumber(text, code, language) {
  const mark = code ? "." : decimalMark(language);
  const escaped = mark.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  if (!new RegExp(`^-?\\d+(${escaped}\\d+)?$`).test(text)) return null;
  return Number(text.replace(mark, "."));
}

function splitList(text, language) {
  const { or, and } = wordsOf.get(language);
  return text.split(new RegExp(`\\s*,\\s*(?:(?:${or}|${and})\\s+)?|\\s+(?:${or}|${and})\\s+`)).map((item) => unwrap(item).text);
}

export function formatValue(value, language) {
  if (Array.isArray(value)) return value.map((item) => formatValue(item, language)).join(", ");
  if (typeof value === "number") {
    return new Intl.NumberFormat(language, { maximumFractionDigits: 3, useGrouping: false }).format(value);
  }
  return String(value);
}

/** `null` if the marked text shows `expected`, else what is wrong. */
export function mismatch(expected, raw, language) {
  const { text, code } = unwrap(raw);
  if (Array.isArray(expected)) {
    const items = splitList(text, language);
    return JSON.stringify(items) === JSON.stringify(expected.map(String)) ? null : `lists [${items.join(", ")}]`;
  }
  if (typeof expected === "number") {
    const shown = parseNumber(text, code, language);
    if (shown === null) return `shows "${text}", which is not a number in ${language} notation`;
    return Math.abs(shown - expected) < 1e-9 ? null : `shows ${text}`;
  }
  return text === String(expected) ? null : `shows "${text}"`;
}
