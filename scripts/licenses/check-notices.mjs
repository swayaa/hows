// Compares the shipped license inventory with THIRD-PARTY-NOTICES.md.
// Missing, unknown, or incompatible licenses fail the process.
//
// Rust: `cargo tree -e normal,no-proc-macro` for the core workspace and the
// Tauri workspace, target x86_64-pc-windows-msvc. That is the graph linked
// into hows.exe and steps-cli. Proc-macro crates and dependencies that exist
// only because Cargo unifies build-script features stay out.
//
// npm: `npm ls --omit=dev --all` from app/, including Svelte because
// @lucide/svelte requires it. A node without a version is an unmet peer
// and is not an installed package. Bare imports under app/src are included
// as well.

import { execFileSync } from 'node:child_process';
import { readdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..', '..');
const noticesPath = join(root, 'THIRD-PARTY-NOTICES.md');
const platform = 'x86_64-pc-windows-msvc';

const allowed = new Set([
  '0BSD',
  'Apache-2.0',
  'BSD-2-Clause',
  'BSD-3-Clause',
  'CC0-1.0',
  'ISC',
  'MIT',
  'MPL-2.0',
  'OFL-1.1',
  'Unicode-3.0',
  'Unicode-DFS-2016',
  'Unlicense',
  'Zlib',
]);

const firstParty = new Set([
  'hows',
  'hows_lib',
  'steps-capture',
  'steps-cli',
  'steps-export',
  'steps-i18n',
  'steps-session',
  'steps-store',
]);

/** Cargo writes dual licenses as `MIT/Apache-2.0` or `Apache-2.0 / MIT`. */
function spdx(expression) {
  return expression.replace(/\s*\/\s*/g, ' OR ');
}

/** @param {string} expression */
function classify(expression) {
  const text = spdx(expression).trim();
  if (!text) return 'unknown';
  const clauses = text.split(/\s+AND\s+/i);
  for (const clause of clauses) {
    const options = clause
      .replace(/[()]/g, ' ')
      .split(/\s+OR\s+/i)
      .map((part) => part.trim())
      .filter(Boolean);
    if (options.length === 0) return 'unknown';
    const forbidden = options.filter((option) => /^(AGPL|GPL|LGPL|SSPL)-/i.test(option));
    if (forbidden.length === options.length) return 'incompatible';
    if (!options.some((option) => allowed.has(option))) return 'unknown';
  }
  return 'ok';
}

function cargoRuntime(manifest) {
  const metadata = JSON.parse(execFileSync(
    'cargo',
    ['metadata', '--locked', '--format-version', '1', '--filter-platform', platform, '--manifest-path', manifest],
    { cwd: root, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 },
  ));
  const licenses = new Map(metadata.packages.map((pkg) => [`${pkg.name} ${pkg.version}`, pkg.license ?? null]));
  const tree = execFileSync(
    'cargo',
    ['tree', '--locked', '--manifest-path', manifest, '-e', 'normal,no-proc-macro', '--target', platform, '--prefix', 'none', '--format', '{p}'],
    { cwd: root, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 },
  );
  /** @type {{ name: string, version: string, license: string | null, origin: string }[]} */
  const found = [];
  const seen = new Set();
  for (const line of tree.split(/\r?\n/)) {
    const match = /^(\S+) v(\S+)/.exec(line);
    if (!match) continue;
    const key = `${match[1]} ${match[2]}`;
    if (seen.has(key)) continue;
    seen.add(key);
    found.push({
      name: match[1],
      version: match[2],
      license: licenses.get(key) ?? null,
      origin: manifest,
    });
  }
  return found;
}

function lockEntry(lock, name) {
  return lock.packages?.[`node_modules/${name}`] ?? null;
}

function sourceImports() {
  const dir = join(root, 'app', 'src');
  /** @type {string[]} */
  const names = [];
  const visit = (path) => {
    for (const entry of readdirSync(path)) {
      const full = join(path, entry);
      if (statSync(full).isDirectory()) {
        visit(full);
        continue;
      }
      if (!/\.(svelte|ts|js)$/.test(entry)) continue;
      const text = readFileSync(full, 'utf8');
      for (const match of text.matchAll(/from\s+['"]([^'"]+)['"]/g)) {
        const spec = match[1];
        if (spec.startsWith('.') || spec.startsWith('$') || spec.startsWith('node:')) continue;
        const name = spec.startsWith('@') ? spec.split('/').slice(0, 2).join('/') : spec.split('/')[0];
        names.push(name);
      }
    }
  };
  visit(dir);
  return names;
}

function npmProduction() {
  const npm = process.platform === 'win32' ? 'npm.cmd' : 'npm';
  const raw = execFileSync(npm, ['ls', '--omit=dev', '--all', '--json'], {
    cwd: join(root, 'app'),
    encoding: 'utf8',
    maxBuffer: 16 * 1024 * 1024,
    shell: process.platform === 'win32',
  });
  const lock = JSON.parse(readFileSync(join(root, 'app', 'package-lock.json'), 'utf8'));
  /** @type {{ name: string, version: string, license: string | null, origin: string }[]} */
  const found = [];
  const seen = new Set();
  const add = (name, version, origin) => {
    const key = `${name} ${version}`;
    if (seen.has(key)) return;
    seen.add(key);
    const entry = lockEntry(lock, name);
    const sameVersion = entry && entry.version === version;
    found.push({
      name,
      version,
      license: sameVersion && typeof entry.license === 'string' ? entry.license : null,
      origin,
    });
  };
  const walk = (node) => {
    for (const [name, child] of Object.entries(node.dependencies ?? {})) {
      if (!child || typeof child !== 'object') continue;
      if (typeof child.version === 'string' && child.version.length > 0) add(name, child.version, 'npm ls --omit=dev');
      walk(child);
    }
  };
  walk(JSON.parse(raw));
  for (const name of sourceImports()) {
    const entry = lockEntry(lock, name);
    if (!entry || typeof entry.version !== 'string') {
      throw new Error(`app/src imports ${name}, which has no lockfile entry.`);
    }
    add(name, entry.version, 'app/src import');
  }
  return found;
}

const notices = readFileSync(noticesPath, 'utf8');
const listed = new Set();
for (const match of notices.matchAll(/`([^`\s]+)\s+([^`\n]+)`/g)) {
  listed.add(`${match[1]} ${match[2].trim()}`);
}

const inventory = [
  ...cargoRuntime('core/Cargo.toml'),
  ...cargoRuntime('app/src-tauri/Cargo.toml'),
  ...npmProduction(),
];

/** @type {Map<string, { name: string, version: string, license: string | null, origin: string }>} */
const unique = new Map();
for (const item of inventory) {
  unique.set(`${item.name} ${item.version}`, item);
}

const missing = [];
const unknown = [];
const incompatible = [];
const firstPartyProblems = [];

for (const item of unique.values()) {
  const key = `${item.name} ${item.version}`;
  const verdict = classify(item.license ?? '');
  if (firstParty.has(item.name)) {
    if (verdict !== 'ok' || item.license !== 'MIT') {
      firstPartyProblems.push(`${key} license ${item.license ?? '(none)'} from ${item.origin}`);
    }
    continue;
  }
  if (verdict === 'incompatible') incompatible.push(`${key} license ${item.license} from ${item.origin}`);
  else if (verdict === 'unknown') unknown.push(`${key} license ${item.license ?? '(none)'} from ${item.origin}`);
  if (!listed.has(key)) missing.push(`${key} from ${item.origin}`);
}

console.log(`Inventory ${unique.size} packages. Notices name ${listed.size} versioned components.`);
if (missing.length === 0 && unknown.length === 0 && incompatible.length === 0 && firstPartyProblems.length === 0) {
  console.log('License inventory matches THIRD-PARTY-NOTICES.md.');
  process.exit(0);
}
for (const line of incompatible) console.error(`incompatible: ${line}`);
for (const line of unknown) console.error(`unknown: ${line}`);
for (const line of missing) console.error(`missing from THIRD-PARTY-NOTICES.md: ${line}`);
for (const line of firstPartyProblems) console.error(`first-party: ${line}`);
process.exit(1);
