// Relative links and anchors in the docs resolve inside the public tree, and
// every wiki page is linked from somewhere. External URLs are not fetched.

import { existsSync, statSync } from "node:fs";
import path from "node:path";
import { anchorsIn, classify, docFiles, linksIn, publicFiles, readDoc, report, repoRoot } from "./lib.mjs";

const files = publicFiles();
const publicSet = new Set(files);
const docs = docFiles(files);
const anchorCache = new Map();
const linkedPages = new Set();
const findings = [];

function anchorsOf(file) {
  if (!anchorCache.has(file)) anchorCache.set(file, anchorsIn(readDoc(file)));
  return anchorCache.get(file);
}

function isPublicDirectory(relative) {
  const prefix = relative === "" ? "" : `${relative}/`;
  return files.some((file) => file.startsWith(prefix));
}

for (const file of docs) {
  for (const { target, line } of linksIn(readDoc(file))) {
    if (/^[a-z][a-z0-9+.-]*:/i.test(target) || target.startsWith("//")) continue;

    const hash = target.indexOf("#");
    const rawPath = hash < 0 ? target : target.slice(0, hash);
    const anchor = hash < 0 ? "" : decodeURIComponent(target.slice(hash + 1));
    let decoded;
    try {
      decoded = decodeURIComponent(rawPath);
    } catch {
      findings.push({ file, line, message: `malformed link "${target}"` });
      continue;
    }

    const resolved =
      decoded === ""
        ? file
        : decoded.startsWith("/")
          ? path.posix.normalize(decoded.slice(1))
          : path.posix.normalize(path.posix.join(path.posix.dirname(file), decoded));
    const relative = resolved.replace(/\/$/, "");

    if (relative === ".." || relative.startsWith("../")) {
      findings.push({ file, line, message: `link leaves the repository: "${target}"` });
      continue;
    }
    const absolute = path.join(repoRoot, relative);
    if (!existsSync(absolute)) {
      findings.push({ file, line, message: `broken link "${target}": ${relative} does not exist` });
      continue;
    }
    const directory = statSync(absolute).isDirectory();
    if (directory ? !isPublicDirectory(relative === "." ? "" : relative) : !publicSet.has(relative)) {
      findings.push({ file, line, message: `link to a file outside the public tree: "${target}"` });
      continue;
    }
    if (!directory && relative !== file) linkedPages.add(relative);

    if (anchor !== "") {
      if (directory || !relative.toLowerCase().endsWith(".md")) {
        findings.push({ file, line, message: `anchor on a non-Markdown target: "${target}"` });
      } else if (!anchorsOf(relative).has(anchor)) {
        findings.push({ file, line, message: `missing anchor "#${anchor}" in ${relative}` });
      }
    }
  }
}

for (const doc of docs.map(classify)) {
  if (doc.tree && !linkedPages.has(doc.file)) {
    findings.push({ file: doc.file, message: "no other page links here" });
  }
}

report(findings, `Link check passed: ${docs.length} pages, no broken relative links.`);
