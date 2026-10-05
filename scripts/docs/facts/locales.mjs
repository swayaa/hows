// Texts in each documented language: `ui.<path>` from the app's locale files
// (app/src/locales), `i18n.<key>` from the core locales (core/i18n/locales),
// and `i18n.languages`, the language names in the order the app lists them.

import path from "node:path";
import { pathToFileURL } from "node:url";
import { languages, repoRoot } from "../lib.mjs";
import { read, readJson } from "./source.mjs";

const uiIndex = "app/src/locales/index.ts";
const coreLocales = "core/i18n/locales";
const coreIndex = "core/i18n/src/lib.rs";

function flatten(object, prefix, into = {}) {
  for (const [key, value] of Object.entries(object)) {
    if (value && typeof value === "object") flatten(value, `${prefix}.${key}`, into);
    else into[`${prefix}.${key}`] = value;
  }
  return into;
}

/** Turns `{ en: { k: v }, de: { k: w } }` into one fact per key. */
function perLanguage(tables, source) {
  const keys = new Set(Object.values(tables).flatMap(Object.keys));
  return [...keys].map((key) => ({
    key,
    byLanguage: Object.fromEntries(languages.map((language) => [language, tables[language][key]])),
    source,
  }));
}

export default async function localeFacts() {
  if (!process.features.typescript) {
    throw new Error(`reading ${uiIndex} needs Node.js with TypeScript type stripping (22.18 or newer)`);
  }
  const { UI_LOCALES } = await import(pathToFileURL(path.join(repoRoot, uiIndex)).href);
  const ui = {};
  const core = {};
  for (const language of languages) {
    const locale = UI_LOCALES.find((candidate) => candidate.code === language);
    if (!locale) throw new Error(`${uiIndex}: no locale "${language}"`);
    ui[language] = flatten(locale.messages, "ui");
    core[language] = flatten(readJson(`${coreLocales}/${language}.json`), "i18n");
  }

  const order = [...read(coreIndex).matchAll(/code: "([^"]+)"/g)].map((found) => found[1]);
  const names = order.map((code) => readJson(`${coreLocales}/${code}.json`)["locale.name"]);

  return [
    ...perLanguage(ui, "app/src/locales"),
    ...perLanguage(core, coreLocales),
    { key: "i18n.languages", value: names, source: coreIndex },
  ];
}
