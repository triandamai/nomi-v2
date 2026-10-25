#!/usr/bin/env node
// Picks the next version for Relay packages and writes it into them.
//
// Both packages share one version. The next one is:
//   - the version in the package.json files, when it's ahead of what npm has (a deliberate
//     minor or major bump made by editing package.json);
//   - otherwise the highest version npm has, with its patch number raised by one.
//
// Prints the chosen version on stdout. With --dry-run it changes nothing. With --set x.y.z it
// skips the choice and writes that version (the release workflow uses this to record the
// version it just published onto the latest main).
import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '..');
const PACKAGES = ['sdk/js', 'cli'];
const VERSION_FILE = join(root, 'sdk/js/src/version.ts');
const SEMVER = /^(\d+)\.(\d+)\.(\d+)$/;

const dryRun = process.argv.includes('--dry-run');
const setIndex = process.argv.indexOf('--set');
const forced = setIndex === -1 ? null : process.argv[setIndex + 1];

function parse(version) {
  const match = SEMVER.exec(version);
  if (!match) throw new Error(`not a plain x.y.z version: ${version}`);
  return match.slice(1).map(Number);
}

function compare(a, b) {
  const [x, y] = [parse(a), parse(b)];
  for (let i = 0; i < x.length; i++) if (x[i] !== y[i]) return x[i] - y[i];
  return 0;
}

const max = (versions) => versions.reduce((best, version) => (best && compare(best, version) >= 0 ? best : version), null);

function published(name) {
  try {
    return execFileSync('npm', ['view', name, 'version'], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim() || null;
  } catch (error) {
    if (String(error.stderr).includes('E404')) return null; // never published
    throw error;
  }
}

const manifests = PACKAGES.map((dir) => {
  const path = join(root, dir, 'package.json');
  return { path, json: JSON.parse(readFileSync(path, 'utf8')) };
});

function choose() {
  const local = max(manifests.map(({ json }) => json.version));
  const remote = max(manifests.map(({ json }) => published(json.name)).filter(Boolean));
  if (!remote || compare(local, remote) > 0) return local;
  const [major, minor, patch] = parse(remote);
  return `${major}.${minor}.${patch + 1}`;
}

const next = forced ?? choose();
parse(next);

if (!dryRun) {
  for (const { path, json } of manifests) {
    json.version = next;
    writeFileSync(path, JSON.stringify(json, null, 2) + '\n');
  }
  writeFileSync(VERSION_FILE, `export const VERSION = '${next}';\n`);
}

console.log(next);
