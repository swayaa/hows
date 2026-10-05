// Reading helpers for the fact sources: repo files, Rust literals, constants,
// and enums. Parsing is deliberately narrow; anything it cannot read throws,
// so a changed code shape fails loudly instead of passing silently.

import { readdirSync, readFileSync } from "node:fs";
import path from "node:path";
import { repoRoot } from "../lib.mjs";

/** Absolute path of a repo-relative path; throws for anything outside the repo. */
function inRepo(relative) {
  const absolute = path.resolve(repoRoot, relative);
  const back = path.relative(repoRoot, absolute);
  if (back.startsWith("..") || path.isAbsolute(back)) throw new Error(`${relative}: outside the repository`);
  return absolute;
}

export function read(file) {
  return readFileSync(inRepo(file), "utf8").replace(/\r\n?/g, "\n");
}

export function readJson(file) {
  return JSON.parse(read(file));
}

/** First capture group of `pattern` in `file`, or an error naming both. */
export function match(file, pattern) {
  const found = pattern.exec(read(file));
  if (!found) throw new Error(`${file}: pattern ${pattern} not found`);
  return found[1];
}

/** Body between the braces that open right after `head`, nesting included. */
export function block(file, head) {
  const text = read(file);
  const start = text.indexOf(head);
  if (start < 0) throw new Error(`${file}: "${head}" not found`);
  const open = text.indexOf("{", start + head.length - 1);
  let depth = 0;
  for (let index = open; index < text.length; index++) {
    if (text[index] === "{") depth++;
    if (text[index] === "}" && --depth === 0) return text.slice(open + 1, index);
  }
  throw new Error(`${file}: unbalanced braces after "${head}"`);
}

/** A Rust string, integer, float, or bool literal; `undefined` for anything else. */
export function rustLiteral(expression) {
  const text = expression.trim();
  const string = /^"((?:[^"\\]|\\.)*)"$/.exec(text);
  if (string) return JSON.parse(`"${string[1]}"`);
  if (/^-?\d[\d_]*(\.\d[\d_]*)?$/.test(text)) return Number(text.replace(/_/g, ""));
  if (text === "true" || text === "false") return text === "true";
  return undefined;
}

function rustFiles(directory) {
  return readdirSync(inRepo(directory), { withFileTypes: true }).flatMap((entry) => {
    const relative = `${directory}/${entry.name}`;
    if (entry.isDirectory()) return entry.name === "target" ? [] : rustFiles(relative);
    return entry.name.endsWith(".rs") ? [relative] : [];
  });
}

let constants;

/** Value of a `pub const NAME: T = literal;` anywhere in the Rust sources. */
export function rustConstant(name) {
  constants ??= new Map(
    ["core", "app/src-tauri/src"]
      .flatMap(rustFiles)
      .flatMap((file) =>
        [...read(file).matchAll(/^\s*pub const (\w+): [^=]+= ([^;]+);/gm)].map((found) => [
          found[1],
          { file, value: rustLiteral(found[2]) },
        ]),
      ),
  );
  const constant = constants.get(name);
  if (!constant || constant.value === undefined) throw new Error(`Rust constant ${name} not found or not a literal`);
  return constant;
}

/** Variant names of `pub enum NAME` in declaration order, with the `#[default]` one. */
export function rustEnum(file, name) {
  const variants = [];
  let fallback = null;
  let marked = false;
  for (const line of block(file, `pub enum ${name} `).split("\n")) {
    const trimmed = line.trim();
    if (trimmed === "#[default]") marked = true;
    const variant = /^([A-Z]\w*)\s*[,({]?/.exec(trimmed);
    if (!variant) continue;
    variants.push(variant[1]);
    if (marked) fallback = variant[1];
    marked = false;
  }
  return { variants, fallback };
}

export function snakeCase(name) {
  return name.replace(/(?<=[a-z0-9])([A-Z])/g, "_$1").toLowerCase();
}
