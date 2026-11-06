import { afterEach, describe, expect, it, vi } from 'vitest';
import { zipSync, strToU8 } from 'fflate';
import { excluded, importGithub, importZip, readZip, uploadFiles } from './projectImport';

afterEach(() => vi.unstubAllGlobals());

function backend() {
	const bodies: { files: { path: string }[]; finish: boolean }[] = [];
	vi.stubGlobal(
		'fetch',
		vi.fn(async (url: string, init: RequestInit) => {
			if (String(url).endsWith('import/github')) return new Response(JSON.stringify({ imported: 3, skipped: 0, stack: 'svelte' }));
			const body = JSON.parse(String(init.body));
			bodies.push(body);
			return new Response(JSON.stringify({ imported: body.files.length, skipped: 0, stack: body.finish ? 'sveltekit' : '' }));
		}),
	);
	return bodies;
}

describe('project import', () => {
	it('leaves out dependencies, build output and secrets', () => {
		expect(excluded('node_modules/x/index.js')).toBe(true);
		expect(excluded('src/app.ts')).toBe(false);
	});

	it('reads a zip, dropping its single top folder and what can’t be imported', () => {
		const zip = zipSync({
			'repo/package.json': strToU8('{}'),
			'repo/src/app.ts': strToU8('export {}'),
			'repo/node_modules/a.js': strToU8('x'),
			'repo/logo.bin': new Uint8Array([0, 1, 2, 0]),
			'repo/': new Uint8Array(),
		});
		const { files, skipped } = readZip(zip);
		expect(files.map((f) => f.path).sort()).toEqual(['package.json', 'src/app.ts']);
		expect(skipped).toBeGreaterThanOrEqual(2);
	});

	it('uploads in batches and finishes on the last one', async () => {
		const bodies = backend();
		const files = Array.from({ length: 450 }, (_, i) => ({ path: `f${i}.txt`, content: 'x' }));
		const progress: number[] = [];
		const result = await uploadFiles('p1', files, (sent) => progress.push(sent));
		expect(bodies.length).toBe(2);
		expect(bodies.map((b) => b.finish)).toEqual([false, true]);
		expect(result).toEqual({ imported: 450, skipped: 0, stack: 'sveltekit' });
		expect(progress.at(-1)).toBe(450);
	});

	it('imports a zip file, refusing an empty one, and a GitHub repo', async () => {
		backend();
		const zip = zipSync({ 'index.html': strToU8('<h1>Hi</h1>'), 'node_modules/x.js': strToU8('x') });
		const file = new File([zip], 'site.zip', { type: 'application/zip' });
		const result = await importZip('p1', file);
		expect(result).toMatchObject({ imported: 1, skipped: 1 });
		const empty = new File([zipSync({})], 'empty.zip');
		await expect(importZip('p1', empty)).rejects.toThrow('empty');
		expect(await importGithub('p1', 'nomi/app')).toMatchObject({ stack: 'svelte' });
	});
});
