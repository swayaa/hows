// Defaults, limits, shortcuts, formats, and messages in the docs match the
// code. The docs mark such values with `<!-- fact:key -->…<!-- /fact -->`
// (see facts/markers.mjs); facts/index.mjs reads the values from the code.
// Every translation marks the same facts, and required facts such as each
// error code and each settings limit appear in every language.

import { classify, counterpart, docFiles, languages, readDoc, report, sourceLanguage } from "./lib.mjs";
import { expectedValue, formatValue, mismatch } from "./facts/display.mjs";
import { loadFacts } from "./facts/index.mjs";
import { markersIn } from "./facts/markers.mjs";

const { facts, required } = await loadFacts();

function valueOf(key, language) {
  const fact = facts.get(key);
  if (!fact) return undefined;
  if (fact.error) throw new Error(fact.error);
  return fact.byLanguage ? fact.byLanguage[language] : fact.value;
}

const docs = docFiles().map(classify);
const findings = [];
const marked = new Map();
const covered = new Map(languages.map((language) => [language, new Set()]));
let checked = 0;

for (const doc of docs) {
  const { markers, exempt, problems } = markersIn(readDoc(doc.file));
  const specs = new Set();
  marked.set(doc.file, specs);
  for (const problem of problems) findings.push({ file: doc.file, ...problem });

  for (const marker of markers) {
    specs.add(marker.spec);
    covered.get(doc.language).add(marker.key);
    const fact = facts.get(marker.key);
    if (!fact) {
      findings.push({ file: doc.file, line: marker.line, message: `unknown fact "${marker.key}"` });
      continue;
    }
    let expected;
    try {
      expected = expectedValue(marker, doc.language, valueOf);
    } catch (error) {
      findings.push({ file: doc.file, line: marker.line, message: `${marker.spec}: ${error.message}` });
      continue;
    }
    checked++;
    const wrong = mismatch(expected, marker.text, doc.language);
    if (wrong) {
      findings.push({
        file: doc.file,
        line: marker.line,
        message: `${marker.spec} ${wrong}, but ${fact.source} says "${formatValue(expected, doc.language)}"`,
      });
    }
  }

  for (const { key, line } of exempt) {
    specs.add(`exempt:${key}`);
    covered.get(doc.language).add(key);
    if (!required.some((entry) => entry.key === key)) {
      findings.push({ file: doc.file, line, message: `fact-exempt "${key}" is not a required fact` });
    }
  }
}

for (const doc of docs) {
  if (doc.language === sourceLanguage) continue;
  const original = counterpart(doc, sourceLanguage);
  const theirs = marked.get(original);
  if (!theirs) continue;
  const ours = marked.get(doc.file);
  for (const spec of theirs) {
    if (!ours.has(spec)) findings.push({ file: doc.file, message: `${original} marks "${spec}", this translation does not` });
  }
  for (const spec of ours) {
    if (!theirs.has(spec)) findings.push({ file: doc.file, message: `marks "${spec}", ${original} does not` });
  }
}

for (const language of languages) {
  for (const { key, source } of required) {
    if (!covered.get(language).has(key)) {
      findings.push({
        file: source,
        message: `"${key}" appears in no ${language} page; mark its text with a fact marker or list it in <!-- fact-exempt: … -->`,
      });
    }
  }
}

report(findings, `Facts check passed: ${checked} marked values in ${docs.length} pages match the code.`);
