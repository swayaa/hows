// Shared helpers for the documentation checks. No dependencies, no network.
//
// Which files count as documentation: every Markdown file git knows about
// (tracked or untracked but not ignored), minus `excludePrefixes` and minus
// files marked `export-ignore` in .gitattributes, which stay internal and
// monolingual.
//
// Language pairing:
// - `<treeRoot>/<lang>/<path>.md` pairs with `<treeRoot>/<other>/<path>.md`.
// - Everywhere else `NAME.md` is the first language and `NAME.<lang>.md`
//   holds each other language.

import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));

export const repoRoot = path.resolve(here, "..", "..");

export const config = JSON.parse(readFileSync(path.join(here, "config.json"), "utf8"));

export const languages = config.languages.map((language) => language.code);
export const sourceLanguage = languages[0];

const labelOf = new Map(config.languages.map((language) => [language.code, language.label]));

function git(args, input) {
  return execFileSync("git", ["-C", repoRoot, ...args], {
    encoding: "utf8",
    input,
    maxBuffer: 16 * 1024 * 1024,
  });
}

function exportIgnored(files) {
  if (files.length === 0) return new Set();
  const output = git(["check-attr", "export-ignore", "--stdin"], files.join("\n") + "\n");
  const ignored = new Set();
  for (const line of output.split("\n")) {
    const match = /^(.*): export-ignore: (.*)$/.exec(line);
    if (match && match[2] === "set") ignored.add(match[1]);
  }
  return ignored;
}

/** All files of the public tree, repo-relative with `/` separators. */
export function publicFiles() {
  // A file with merge conflicts is listed once per stage.
  const listed = [...new Set(git(["ls-files", "--cached", "--others", "--exclude-standard"]).split("\n"))]
    .filter(Boolean)
    .filter((file) => existsSync(path.join(repoRoot, file)));
  const ignored = exportIgnored(listed);
  return listed.filter((file) => !ignored.has(file));
}

/** Documentation files, sorted. */
export function docFiles(files = publicFiles()) {
  return files
    .filter((file) => file.toLowerCase().endsWith(".md"))
    .filter((file) => !config.excludePrefixes.some((prefix) => file.startsWith(prefix)))
    .sort();
}

/**
 * Language and language-neutral key of a doc file. Files with the same key
 * are translations of each other.
 */
export function classify(file) {
  const parts = file.split("/");
  const treeParts = config.treeRoot.split("/");
  const inTree =
    parts.length > treeParts.length + 1 &&
    treeParts.every((part, index) => parts[index] === part) &&
    languages.includes(parts[treeParts.length]);
  if (inTree) {
    const language = parts[treeParts.length];
    const rest = parts.slice(treeParts.length + 1).join("/");
    return { file, language, key: `tree:${rest}`, tree: true };
  }
  const match = /^(.*)\.([a-z]{2})\.md$/i.exec(file);
  if (match && languages.includes(match[2]) && match[2] !== sourceLanguage) {
    return { file, language: match[2], key: `file:${match[1]}.md`, tree: false };
  }
  return { file, language: sourceLanguage, key: `file:${file}`, tree: false };
}

/** Repo-relative path of the version of `doc` in `language`. */
export function counterpart(doc, language) {
  if (doc.tree) {
    const rest = doc.key.slice("tree:".length);
    return `${config.treeRoot}/${language}/${rest}`;
  }
  const base = doc.key.slice("file:".length);
  return language === sourceLanguage ? base : base.replace(/\.md$/i, `.${language}.md`);
}

/** Relative link from one repo file to another, with `/` separators. */
export function relativeLink(from, to) {
  const relative = path.posix.relative(path.posix.dirname(from), to);
  return relative === "" ? path.posix.basename(to) : relative;
}

/** The line every doc page starts with, e.g. `English | [Deutsch](README.de.md)`. */
export function languageLine(doc) {
  return languages
    .map((language) =>
      language === doc.language
        ? labelOf.get(language)
        : `[${labelOf.get(language)}](${relativeLink(doc.file, counterpart(doc, language))})`,
    )
    .join(" | ");
}

export function readDoc(file) {
  return readFileSync(path.join(repoRoot, file), "utf8").replace(/\r\n?/g, "\n");
}

/**
 * The text with fenced code blocks and inline code blanked out. Line and
 * column positions stay the same, so findings point at the real source.
 */
export function withoutCode(text) {
  const blank = (chunk) => chunk.replace(/[^\n]/g, " ");
  let fence = null;
  const lines = text.split("\n").map((line) => {
    const marker = /^ {0,3}(`{3,}|~{3,})/.exec(line)?.[1];
    if (fence) {
      if (marker && marker[0] === fence[0] && marker.length >= fence.length && line.trim() === marker) {
        fence = null;
      }
      return blank(line);
    }
    if (marker) {
      fence = marker;
      return blank(line);
    }
    return line.replace(/(?<!`)(`+)(?!`)[^\n]*?(?<!`)\1(?!`)/g, blank);
  });
  return lines.join("\n");
}

export function lineOf(text, index) {
  let line = 1;
  for (let i = 0; i < index; i++) if (text.charCodeAt(i) === 10) line++;
  return line;
}

/** Link targets with their line number: inline links, images, references, HTML. */
export function linksIn(text) {
  const source = withoutCode(text);
  const links = [];
  const add = (target, index) => {
    let cleaned = target.trim();
    if (cleaned.startsWith("<") && cleaned.endsWith(">")) cleaned = cleaned.slice(1, -1);
    links.push({ target: cleaned, line: lineOf(source, index) });
  };
  for (const match of source.matchAll(/!?\[(?:[^\[\]]|\[[^\]]*\])*\]\(\s*(<[^>]*>|[^\s)]+)(?:\s+(?:"[^"]*"|'[^']*'))?\s*\)/g)) {
    add(match[1], match.index);
  }
  for (const match of source.matchAll(/^ {0,3}\[[^\]]+\]:\s*(<[^>]*>|\S+)/gm)) {
    add(match[1], match.index);
  }
  for (const match of source.matchAll(/<(?:a|img)\b[^>]*?\s(?:href|src)\s*=\s*["']([^"']+)["']/gi)) {
    add(match[1], match.index);
  }
  return links;
}

/** GitHub's heading slug: lower case, punctuation dropped, spaces to hyphens. */
export function slugify(heading) {
  return heading
    .replace(/!\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/\[([^\]]*)\]\([^)]*\)/g, "$1")
    .replace(/<[^>]+>/g, "")
    .replace(/[`*]/g, "")
    .trim()
    .toLowerCase()
    .replace(/[^\p{L}\p{M}\p{N}\p{Pc} -]/gu, "")
    .replace(/ /g, "-");
}

/** Every anchor a Markdown file offers, with GitHub's `-1`, `-2` suffixes for repeats. */
export function anchorsIn(text) {
  const source = withoutCode(text);
  const anchors = new Set();
  const seen = new Map();
  for (const match of source.matchAll(/^ {0,3}#{1,6}[ \t]+(.+?)[ \t#]*$/gm)) {
    const slug = slugify(match[1]);
    const count = seen.get(slug) ?? 0;
    seen.set(slug, count + 1);
    anchors.add(count === 0 ? slug : `${slug}-${count}`);
  }
  for (const match of source.matchAll(/<a\b[^>]*?\s(?:id|name)\s*=\s*["']([^"']+)["']/gi)) {
    anchors.add(match[1]);
  }
  return anchors;
}

/** Prints findings as `file:line: message` (and as annotations on GitHub Actions). */
export function report(findings, okMessage) {
  const annotate = process.env.GITHUB_ACTIONS === "true";
  for (const finding of findings) {
    const where = finding.line ? `${finding.file}:${finding.line}` : finding.file;
    console.error(`${where}: ${finding.message}`);
    if (annotate) {
      const line = finding.line ? `,line=${finding.line}` : "";
      console.log(`::error file=${finding.file}${line}::${finding.message}`);
    }
  }
  if (findings.length > 0) {
    console.error(`\n${findings.length} problem(s) found.`);
    process.exitCode = 1;
  } else {
    console.log(okMessage);
  }
}
