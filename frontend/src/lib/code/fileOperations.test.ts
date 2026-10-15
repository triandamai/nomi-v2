import { describe, expect, it } from 'vitest';
import { zipSync, strToU8 } from 'fflate';
import {
	normalizePath,
	parentFolder,
	baseName,
	joinPath,
	isSubpathOf,
	relocateFilePath,
	relocateFolderFilePath,
	isZipFile,
	shouldIgnoreZipEntry,
	extractZipArchive,
} from './fileOperations';

describe('fileOperations', () => {
	it('normalizes paths properly', () => {
		expect(normalizePath('///src//components///Button.svelte/')).toBe('src/components/Button.svelte');
		expect(normalizePath('index.html')).toBe('index.html');
		expect(normalizePath('')).toBe('');
		expect(normalizePath('/app/')).toBe('app');
	});

	it('computes parent folders', () => {
		expect(parentFolder('src/components/Button.svelte')).toBe('src/components');
		expect(parentFolder('src/index.ts')).toBe('src');
		expect(parentFolder('index.html')).toBe('');
		expect(parentFolder('nested/a/b/c/d.txt')).toBe('nested/a/b/c');
	});

	it('computes basenames', () => {
		expect(baseName('src/components/Button.svelte')).toBe('Button.svelte');
		expect(baseName('src/components')).toBe('components');
		expect(baseName('index.html')).toBe('index.html');
		expect(baseName('')).toBe('');
	});

	it('joins paths correctly', () => {
		expect(joinPath('src', 'components/Button.svelte')).toBe('src/components/Button.svelte');
		expect(joinPath('', 'index.html')).toBe('index.html');
		expect(joinPath('src', '')).toBe('src');
		expect(joinPath('', '')).toBe('');
		expect(joinPath('src/', '/nested/')).toBe('src/nested');
	});

	it('checks subpaths correctly', () => {
		expect(isSubpathOf('src/components/Button.svelte', 'src')).toBe(true);
		expect(isSubpathOf('src/components', 'src/components')).toBe(true);
		expect(isSubpathOf('src-other/file.txt', 'src')).toBe(false);
		expect(isSubpathOf('index.html', '')).toBe(true);
		expect(isSubpathOf('src/components/Button.svelte', 'src/components')).toBe(true);
		expect(isSubpathOf('src/lib', 'src/lib/server')).toBe(false);
	});

	it('relocates file paths correctly', () => {
		expect(relocateFilePath('src/utils.ts', 'src/lib')).toBe('src/lib/utils.ts');
		expect(relocateFilePath('src/utils.ts', '')).toBe('utils.ts');
		expect(relocateFilePath('utils.ts', 'src')).toBe('src/utils.ts');
		expect(relocateFilePath('deep/nested/path/config.json', 'target')).toBe('target/config.json');
	});

	it('relocates folder files correctly', () => {
		expect(
			relocateFolderFilePath('src/components', 'src/ui', 'src/components/Button.svelte')
		).toBe('src/ui/components/Button.svelte');
		expect(
			relocateFolderFilePath('components', '', 'components/icons/Close.svelte')
		).toBe('components/icons/Close.svelte');
		expect(
			relocateFolderFilePath('src/icons', 'static/assets', 'src/icons/svg/home.svg')
		).toBe('static/assets/icons/svg/home.svg');
	});

	it('identifies zip files by name or mime type', () => {
		expect(isZipFile({ name: 'project.zip' })).toBe(true);
		expect(isZipFile({ name: 'archive.ZIP' })).toBe(true);
		expect(isZipFile({ name: 'archive', type: 'application/zip' })).toBe(true);
		expect(isZipFile({ name: 'archive', type: 'application/x-zip-compressed' })).toBe(true);
		expect(isZipFile({ name: 'code.ts' })).toBe(false);
		expect(isZipFile({ name: 'zip.txt' })).toBe(false);
	});

	it('ignores metadata zip entries', () => {
		expect(shouldIgnoreZipEntry('folder/')).toBe(true);
		expect(shouldIgnoreZipEntry('__MACOSX/._file.txt')).toBe(true);
		expect(shouldIgnoreZipEntry('src/.DS_Store')).toBe(true);
		expect(shouldIgnoreZipEntry('node_modules/pkg/index.js')).toBe(true);
		expect(shouldIgnoreZipEntry('.git/config')).toBe(true);
		expect(shouldIgnoreZipEntry('src/index.js')).toBe(false);
		expect(shouldIgnoreZipEntry('README.md')).toBe(false);
	});

	it('extracts zip archives preserving internal directory structure', () => {
		const zipData = zipSync({
			'package.json': strToU8('{"name":"test"}'),
			'src/': new Uint8Array(0),
			'src/index.ts': strToU8('console.log("hello");'),
			'src/components/App.svelte': strToU8('<script></script>'),
			'__MACOSX/._package.json': strToU8('meta'),
			'node_modules/bad/index.js': strToU8('bad'),
		});

		// Extract to root
		const extractedRoot = extractZipArchive(zipData);
		expect(extractedRoot).toHaveLength(3);
		expect(extractedRoot.map((f) => f.path).sort()).toEqual([
			'package.json',
			'src/components/App.svelte',
			'src/index.ts',
		]);
		expect(extractedRoot.find((f) => f.path === 'package.json')?.content).toBe('{"name":"test"}');

		// Extract to subfolder
		const extractedSub = extractZipArchive(zipData, 'vendor/template');
		expect(extractedSub.map((f) => f.path).sort()).toEqual([
			'vendor/template/package.json',
			'vendor/template/src/components/App.svelte',
			'vendor/template/src/index.ts',
		]);
	});
});
