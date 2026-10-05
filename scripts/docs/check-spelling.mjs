// Spell-checks the docs with cspell: each language with its own dictionaries,
// all with the shared project word list in project-words.txt and German pages
// additionally with project-words.de.txt.

import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { classify, config, docFiles, repoRoot } from "./lib.mjs";

const here = path.dirname(fileURLToPath(import.meta.url));
const cspell = fileURLToPath(import.meta.resolve("cspell/bin.mjs"));
const docs = docFiles().map(classify);
let failed = false;

for (const language of config.languages) {
  const files = docs.filter((doc) => doc.language === language.code).map((doc) => doc.file);
  if (files.length === 0) continue;
  const result = spawnSync(
    process.execPath,
    [
      cspell,
      "lint",
      "--config",
      path.join(here, "cspell.json"),
      "--locale",
      language.spellingLocales ?? language.code,
      "--no-progress",
      "--no-summary",
      "--show-suggestions",
      "--file-list",
      "stdin",
    ],
    { cwd: repoRoot, input: files.join("\n"), stdio: ["pipe", "inherit", "inherit"] },
  );
  if (result.error) throw result.error;
  if (result.status !== 0) {
    failed = true;
  } else {
    console.log(`Spelling check passed for ${language.label}: ${files.length} pages.`);
  }
}

if (failed) {
  console.error(
    "\nFix the typo, or add a correct project term to scripts/docs/project-words.txt" +
      " (all languages) or scripts/docs/project-words.de.txt (German only).",
  );
  process.exitCode = 1;
}
