/** Runs after `vite build`: the built app must not name a remote host. */

import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { filesBelow, remoteAddresses } from "./offline.ts";

const DIST = fileURLToPath(new URL("../dist/", import.meta.url));

const files = filesBelow(DIST, [".html", ".js", ".css"]);
const found = files.flatMap((file) =>
  remoteAddresses(readFileSync(file, "utf8")).map((address) => `${file}: ${address}`),
);

if (files.length === 0) {
  console.error(`check-dist: no files in ${DIST}`);
  process.exit(1);
}
if (found.length > 0) {
  console.error(`check-dist: remote addresses in the build:\n${found.join("\n")}`);
  process.exit(1);
}
console.log(`check-dist: ${files.length} files, no remote addresses`);
