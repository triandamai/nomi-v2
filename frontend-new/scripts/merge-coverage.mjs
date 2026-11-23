// Merges Vitest's Istanbul coverage JSON with Playwright E2E Istanbul JSON files (smoke + backend)
// into a single coverage report: coverage/coverage-final.json, coverage/coverage-summary.json,
// HTML, and a text-summary printed to stdout.
//
// Because Vitest instruments dev-transformed files while Playwright E2E runs production bundles,
// their statement/function/branch maps for the same file can have slightly different offsets.
// E2E files are merged with each other first, then overlaid onto Vitest's base map: files with
// 0 Vitest coverage take the E2E FileCoverage directly, while files already covered by Vitest
// receive E2E line hits without duplicating statement/function/branch map entries.
//
// Usage: node scripts/merge-coverage.mjs <vitest-coverage.json> [e2e1.json e2e2.json ...]
import { existsSync, mkdirSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import libCoverage from 'istanbul-lib-coverage';
import libReport from 'istanbul-lib-report';
import reports from 'istanbul-reports';

const ROOT = process.cwd();
const inputs = process.argv.slice(2);
if (inputs.length === 0) {
	console.error('Usage: node scripts/merge-coverage.mjs <vitest-coverage.json> [e2e1.json ...]');
	process.exit(1);
}

/** Normalises runner-specific absolute paths to ROOT/relative so files from different jobs merge. */
function normalizePath(filePath) {
	const clean = filePath.replace(/\\/g, '/');
	const marker = '/frontend-new/';
	const idx = clean.lastIndexOf(marker);
	if (idx >= 0) return resolve(ROOT, clean.slice(idx + marker.length));
	return resolve(ROOT, clean);
}

function loadMap(input) {
	const full = resolve(ROOT, input);
	if (!existsSync(full)) {
		console.warn(`merge-coverage: skipping missing file ${input}`);
		return null;
	}
	const raw = JSON.parse(readFileSync(full, 'utf8'));
	/** @type {Record<string, any>} */
	const normalized = {};
	for (const [key, data] of Object.entries(raw)) {
		const path = normalizePath(data?.path ?? key);
		normalized[path] = { ...data, path };
	}
	return libCoverage.createCoverageMap(normalized);
}

const [baseArg, ...overlayArgs] = inputs;
const baseMap = loadMap(baseArg);
const e2eMap = libCoverage.createCoverageMap({});
let hasE2e = false;

for (const arg of overlayArgs) {
	const next = loadMap(arg);
	if (!next) continue;
	e2eMap.merge(next);
	hasE2e = true;
}

if (!baseMap && !hasE2e) {
	console.error('merge-coverage: no coverage input files found');
	process.exit(1);
}

const map = baseMap ? libCoverage.createCoverageMap(baseMap.toJSON()) : e2eMap;

if (baseMap && hasE2e) {
	const existing = new Set(map.files());
	for (const file of e2eMap.files()) {
		const efc = e2eMap.fileCoverageFor(file);
		if (!existing.has(file)) {
			map.addFileCoverage(efc);
			continue;
		}
		const ufc = map.fileCoverageFor(file);
		if (ufc.toSummary().statements.covered === 0) {
			map.data[file] = libCoverage.createFileCoverage(efc.toJSON());
			continue;
		}
		const eLines = efc.getLineCoverage();
		const data = ufc.toJSON();
		for (const [id, loc] of Object.entries(data.statementMap)) {
			if (data.s[id] === 0 && (eLines[loc.start.line] ?? 0) > 0) data.s[id] = eLines[loc.start.line];
		}
		for (const [id, fn] of Object.entries(data.fnMap)) {
			const line = fn.line ?? fn.loc?.start?.line;
			if (data.f[id] === 0 && (eLines[line] ?? 0) > 0) data.f[id] = eLines[line];
		}
		for (const [id, br] of Object.entries(data.branchMap)) {
			br.locations.forEach((loc, i) => {
				if (data.b[id][i] === 0 && (eLines[loc.start.line] ?? 0) > 0) data.b[id][i] = eLines[loc.start.line];
			});
		}
		map.data[file] = libCoverage.createFileCoverage(data);
	}
}

const outDir = resolve(ROOT, 'coverage');
mkdirSync(outDir, { recursive: true });

const context = libReport.createContext({
	dir: outDir,
	coverageMap: map,
});

for (const name of ['text-summary', 'json-summary', 'json', 'html']) {
	reports.create(name).execute(context);
}
