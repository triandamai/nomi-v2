// Merges multiple Istanbul coverage JSON files (unit/component + smoke E2E + backend E2E) into a
// single coverage report: coverage/coverage-final.json, coverage/coverage-summary.json, HTML, and
// a text-summary printed to stdout.
//
// Usage: node scripts/merge-coverage.mjs <file1.json> [file2.json ...]
import { existsSync, mkdirSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import libCoverage from 'istanbul-lib-coverage';
import libReport from 'istanbul-lib-report';
import reports from 'istanbul-reports';

const ROOT = process.cwd();
const inputs = process.argv.slice(2);
if (inputs.length === 0) {
	console.error('Usage: node scripts/merge-coverage.mjs <file1.json> [file2.json ...]');
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

const map = libCoverage.createCoverageMap({});
let loadedFiles = 0;

for (const input of inputs) {
	const full = resolve(ROOT, input);
	if (!existsSync(full)) {
		console.warn(`merge-coverage: skipping missing file ${input}`);
		continue;
	}
	const raw = JSON.parse(readFileSync(full, 'utf8'));
	/** @type {Record<string, any>} */
	const normalized = {};
	for (const [key, data] of Object.entries(raw)) {
		const path = normalizePath(data?.path ?? key);
		normalized[path] = { ...data, path };
	}
	map.merge(normalized);
	loadedFiles++;
}

if (loadedFiles === 0) {
	console.error('merge-coverage: no coverage input files found');
	process.exit(1);
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
