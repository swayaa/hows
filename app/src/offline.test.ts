import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { describe, it } from "node:test";
import { NEVER_REQUESTED, filesBelow, remoteAddresses } from "../scripts/offline.ts";

const APP = fileURLToPath(new URL("../", import.meta.url));
const SRC = fileURLToPath(new URL("./", import.meta.url));
const TAURI_CONFIG = `${APP}src-tauri/tauri.conf.json`;

/** The page, styles and code the build starts from. `npm run build` checks the result. */
function sources(): string[] {
  const code = filesBelow(SRC, [".css", ".svelte", ".ts"]).filter((file) => !file.endsWith(".test.ts"));
  return [`${APP}index.html`, ...code];
}

function directives(policy: string): Record<string, string[]> {
  return Object.fromEntries(
    policy
      .split(";")
      .map((directive) => directive.trim().split(/\s+/))
      .filter(([name]) => name)
      .map(([name, ...sources]) => [name, sources]),
  );
}

describe("offline", () => {
  it("names no remote host in index.html, styles and code", () => {
    const found = sources().flatMap((file) =>
      remoteAddresses(readFileSync(file, "utf8")).map((address) => `${file}: ${address}`),
    );
    assert.deepEqual(found, []);
  });

  it("notices stylesheets, fonts and scripts from the web", () => {
    const html = `<link href="https://fonts.googleapis.com/css2?family=Inter" rel="stylesheet" />
      <style>@import url(http://cdn.example/x.css); src: url('https://fonts.gstatic.com/a.woff2')</style>`;
    assert.deepEqual(remoteAddresses(html), [
      "https://fonts.googleapis.com/css2?family=Inter",
      "http://cdn.example/x.css",
      "https://fonts.gstatic.com/a.woff2",
    ]);
  });

  it("notices protocol-relative addresses", () => {
    const html = `<script src="//cdn.example.com/a.js"></script>
      <style>@font-face { src: url(//fonts.gstatic.com/b.woff2) } a { background: url( '//img.example.org:8080/c.png') }</style>
      <img src=//static.example.net/d.svg>`;
    assert.deepEqual(remoteAddresses(html), [
      "//cdn.example.com/a.js",
      "//fonts.gstatic.com/b.woff2",
      "//img.example.org:8080/c.png",
      "//static.example.net/d.svg",
    ]);
  });

  it("does not take comments, local paths or the scheme part of full addresses for hosts", () => {
    const code = `// marks.json holds the defaults
      const re = /a\\/\\/b/; // see core/store
      import x from "./local.ts"; const p = "/assets/font.woff2"; const s = "a//b";
      /* //not a host */ const u = "https://svelte.dev/e/x";`;
    assert.deepEqual(remoteAddresses(code), []);
  });

  it("allows only addresses that are never requested, each with a reason", () => {
    for (const allowed of NEVER_REQUESTED) {
      assert.deepEqual(remoteAddresses(`${allowed.prefix}x`), []);
      assert.ok(allowed.reason.length > 20, allowed.prefix);
    }
  });
});

describe("content security policy", () => {
  it("lets the webview load only the bundle, step images and Tauri's IPC", () => {
    const { csp } = JSON.parse(readFileSync(TAURI_CONFIG, "utf8")).app.security;
    assert.deepEqual(directives(csp), {
      "default-src": ["'none'"],
      "script-src": ["'self'"],
      "style-src": ["'self'"],
      "font-src": ["'self'"],
      // Screenshots and thumbnails arrive as data:image/png URIs from get_step_image and get_guide_thumbnail.
      "img-src": ["data:"],
      // Only for invoke(): on Windows Tauri posts each command to http://ipc.localhost/<command>.
      "connect-src": ["http://ipc.localhost"],
      "base-uri": ["'none'"],
      "form-action": ["'none'"],
    });
  });
});
