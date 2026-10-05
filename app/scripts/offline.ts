/**
 * Finds web addresses the app could load. Hows promises to work without a
 * network (docs/en/privacy.md), so the page, its styles and the build must not
 * name a remote host. Allowed are only addresses that never cause a request.
 */

import { readdirSync, statSync } from "node:fs";
import { join } from "node:path";

type Allowed = { prefix: string; reason: string };

export const NEVER_REQUESTED: readonly Allowed[] = [
  {
    prefix: "http://www.w3.org/",
    reason: "XML namespace names (SVG, XHTML, MathML, XLink) that Svelte and the icons use to create elements. They identify a vocabulary and are never fetched.",
  },
  {
    prefix: "https://svelte.dev/e/",
    reason: "Link inside Svelte error messages, only text for a developer console.",
  },
];

const ADDRESS = /\bhttps?:\/\/[^\s"'`()<>\\]+/g;
/**
 * Protocol-relative addresses (`//host/x`, `url(//host/x)`): a host name with a
 * dot right after a quote, `(` or `=`. Comments (`// text`) and paths stay out.
 */
const RELATIVE_ADDRESS = /(?<=["'`(=]\s*)\/\/[a-z0-9-]+(?:\.[a-z0-9-]+)+(?::\d+)?(?:\/[^\s"'`()<>\\]*)?/gi;

/** Remote addresses in `text`, without the ones in `NEVER_REQUESTED`. */
export function remoteAddresses(text: string): string[] {
  return [...text.matchAll(ADDRESS), ...text.matchAll(RELATIVE_ADDRESS)]
    .sort((a, b) => (a.index ?? 0) - (b.index ?? 0))
    .map((match) => match[0])
    .filter((address) => !NEVER_REQUESTED.some((allowed) => address.startsWith(allowed.prefix)));
}

/** Files below `dir` whose name ends in one of `extensions`. */
export function filesBelow(dir: string, extensions: readonly string[]): string[] {
  return readdirSync(dir).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(path).isDirectory()) return filesBelow(path, extensions);
    return extensions.some((extension) => name.endsWith(extension)) ? [path] : [];
  });
}
