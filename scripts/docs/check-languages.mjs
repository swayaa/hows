// Every doc page exists in every configured language and starts with the
// language line that links its translations.

import { classify, counterpart, docFiles, languageLine, languages, readDoc, report } from "./lib.mjs";

const docs = docFiles().map(classify);
const present = new Set(docs.map((doc) => doc.file));
const findings = [];

for (const doc of docs) {
  for (const language of languages) {
    if (language === doc.language) continue;
    const expected = counterpart(doc, language);
    if (!present.has(expected)) {
      findings.push({ file: doc.file, message: `missing ${language} version: ${expected}` });
    }
  }

  const lines = readDoc(doc.file).split("\n");
  const index = lines.findIndex((line) => line.trim() !== "");
  const first = index < 0 ? "" : lines[index].trim();
  const wanted = languageLine(doc);
  if (first !== wanted) {
    findings.push({
      file: doc.file,
      line: index + 1,
      message: `first line must be the language line "${wanted}", found "${first}"`,
    });
  }
}

report(findings, `Language check passed: ${docs.length} pages in ${languages.join(", ")}.`);
