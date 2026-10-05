// All fact sources. Each returns facts (`{ key, value | byLanguage | error,
// source }`), optionally with `required` keys the docs must cover. To add a
// fact, extend or add a source here; the docs then reference it by key.

import appFacts from "./app.mjs";
import cliFacts from "./cli.mjs";
import errorFacts from "./errors.mjs";
import exportFacts from "./export.mjs";
import localeFacts from "./locales.mjs";
import settingsFacts from "./settings.mjs";
import storeFacts from "./store.mjs";

const sources = [settingsFacts, exportFacts, errorFacts, localeFacts, storeFacts, cliFacts, appFacts];

export async function loadFacts() {
  const facts = new Map();
  const required = [];
  for (const source of sources) {
    const result = await source();
    const list = Array.isArray(result) ? result : result.facts;
    for (const fact of list) {
      if (facts.has(fact.key)) throw new Error(`fact ${fact.key} comes from two sources`);
      facts.set(fact.key, fact);
    }
    for (const key of Array.isArray(result) ? [] : result.required) {
      required.push({ key, source: list[0]?.source });
    }
  }
  for (const { key } of required) {
    if (!facts.has(key)) throw new Error(`required fact ${key} has no source`);
  }
  return { facts, required };
}
