// Converts raw V8 coverage collected during Playwright E2E runs (both browser client chunks and
// Node server.js chunks) into an Istanbul coverage JSON file, remapped through build sourcemaps
// back to the original src/**/*.{ts,svelte}, ws-proxy/**/*.js and server.js files.
//
// Usage: node scripts/e2e-coverage.mjs <v8-dir> <output-json>
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, relative, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import * as acorn from 'acorn';
import convert from 'ast-v8-to-istanbul';
import libCoverage from 'istanbul-lib-coverage';

const ROOT = process.cwd();
const [v8DirArg, outputArg] = process.argv.slice(2);
if (!v8DirArg || !outputArg) {
	console.error('Usage: node scripts/e2e-coverage.mjs <v8-dir> <output-json>');
	process.exit(1);
}

const v8Dir = resolve(ROOT, v8DirArg);
const outputPath = resolve(ROOT, outputArg);

function isTrackedSource(absPath) {
	const rel = relative(ROOT, absPath).replace(/\\/g, '/');
	if (rel.startsWith('..') || rel.startsWith('/')) return false;
	const included = /^src\/.*\.(ts|svelte)$/.test(rel) || /^ws-proxy\/.*\.js$/.test(rel) || rel === 'server.js';
	if (!included) return false;
	if (/^src\/lib\/paraglide\//.test(rel) || /\.test\.(ts|js)$/.test(rel) || /\.d\.ts$/.test(rel)) return false;
	return existsSync(absPath);
}

function resolveScriptPath(url, kind) {
	if (!url) return null;
	if (kind === 'client') {
		try {
			const { pathname } = new URL(url);
			if (!pathname.startsWith('/_app/immutable/')) return null;
			const full = resolve(ROOT, 'build/client', pathname.slice(1));
			return existsSync(full) ? full : null;
		} catch {
			return null;
		}
	}
	if (!url.startsWith('file://')) return null;
	const full = fileURLToPath(url);
	const rel = relative(ROOT, full).replace(/\\/g, '/');
	if (rel.startsWith('..') || rel.includes('node_modules')) return null;
	if (!rel.startsWith('build/') && !rel.startsWith('ws-proxy/') && rel !== 'server.js') return null;
	return existsSync(full) ? full : null;
}

/** @type {Map<string, { code: string, ast: any, sourceMap?: any } | null>} */
const fileCache = new Map();

function loadFile(filePath) {
	if (fileCache.has(filePath)) return fileCache.get(filePath);
	try {
		const code = readFileSync(filePath, 'utf8');
		const ast = acorn.parse(code, { ecmaVersion: 'latest', sourceType: 'module', allowHashBang: true });
		let sourceMap;
		const mapPath = `${filePath}.map`;
		if (existsSync(mapPath)) {
			const rawMap = JSON.parse(readFileSync(mapPath, 'utf8'));
			// adapter-node copies .svelte-kit/output/{client,server} (1 level deeper) into build/.
			const relFromRoot = relative(ROOT, filePath).replace(/\\/g, '/');
			const mapDir = relFromRoot.startsWith('build/client/') || relFromRoot.startsWith('build/server/')
				? dirname(resolve(ROOT, '.svelte-kit/output', relFromRoot.slice('build/'.length)))
				: dirname(filePath);
			if (Array.isArray(rawMap.sources)) {
				rawMap.sources = rawMap.sources.map((src) => {
					if (!src) return src;
					if (src.startsWith('file://')) return fileURLToPath(src);
					return resolve(mapDir, src);
				});
			}
			sourceMap = rawMap;
		}
		const loaded = { code, ast, sourceMap };
		fileCache.set(filePath, loaded);
		return loaded;
	} catch {
		fileCache.set(filePath, null);
		return null;
	}
}

function listJsonFiles(dir) {
	if (!existsSync(dir)) return [];
	return readdirSync(dir)
		.filter((name) => name.endsWith('.json'))
		.map((name) => resolve(dir, name));
}

const merged = libCoverage.createCoverageMap({});
let convertedCount = 0;

for (const kind of ['client', 'server']) {
	const jsonFiles = listJsonFiles(resolve(v8Dir, kind));
	for (const file of jsonFiles) {
		let parsed;
		try {
			parsed = JSON.parse(readFileSync(file, 'utf8'));
		} catch {
			continue;
		}
		const entries = Array.isArray(parsed?.result) ? parsed.result : [];
		for (const entry of entries) {
			const filePath = resolveScriptPath(entry.url, kind);
			if (!filePath || !Array.isArray(entry.functions) || entry.functions.length === 0) continue;
			const loaded = loadFile(filePath);
			if (!loaded) continue;
			try {
				const converted = await convert({
					code: loaded.code,
					ast: loaded.ast,
					sourceMap: loaded.sourceMap,
					coverage: { url: pathToFileURL(filePath).href, functions: entry.functions },
				});
				for (const [sourcePath, data] of Object.entries(converted)) {
					const absPath = sourcePath.startsWith('file://') ? fileURLToPath(sourcePath) : resolve(ROOT, sourcePath);
					if (!isTrackedSource(absPath)) continue;
					const normalized = { ...data, path: absPath };
					merged.merge({ [absPath]: normalized });
					convertedCount++;
				}
			} catch {
				// Skip scripts that fail sourcemap/AST conversion without failing the run.
			}
		}
	}
}

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(outputPath, JSON.stringify(merged.toJSON()));
console.log(`e2e-coverage: wrote ${merged.files().length} tracked files (${convertedCount} script conversions) to ${relative(ROOT, outputPath)}`);
