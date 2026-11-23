import { describe, expect, it } from 'vitest';
import { deviceHref, DEVICE_PATHS, devicePath, diffManifest, parentDir, previewEnv, toFileTree, wantsDevice, watchExit } from './device';

describe('device run', () => {
	it('asks for the device run on the workspace page only', () => {
		expect(deviceHref('p 1')).toBe('/projects/p%201?run=device');
		expect(wantsDevice(new URL('https://app.nomi.build/projects/abc?run=device'))).toBe(true);
		expect(wantsDevice(new URL('https://app.nomi.build/projects/abc'))).toBe(false);
		expect(wantsDevice(new URL('https://app.nomi.build/projects/abc/deploy?run=device'))).toBe(false);
	});

	it('proxies only the runtime endpoints', () => {
		for (const path of ['snapshot', 'manifest', 'files/src/app.ts', 'runtime', 'runtime/check', 'runtime/runs/r-1', 'relay/preview']) expect(DEVICE_PATHS.test(path)).toBe(true);
		for (const path of ['', 'members', 'sandbox/stop', 'runtime/runs/../x', 'secrets']) expect(DEVICE_PATHS.test(path)).toBe(false);
		expect(devicePath('files/src/a b.ts')).toBe('files/src/a%20b.ts');
		expect(devicePath('files/../secrets')).toBeNull();
		expect(devicePath('files/a//b')).toBeNull();
		expect(devicePath('members')).toBeNull();
	});

	it('nests files into a tree and decodes binary ones', () => {
		const tree = toFileTree([
			{ path: 'package.json', content: '{}' },
			{ path: 'src/lib/a.ts', content: 'a' },
			{ path: 'static/x.bin', content: btoa('\x01\x02'), encoding: 'base64' },
		]);
		expect(tree['package.json']).toEqual({ file: { contents: '{}' } });
		expect(tree.src).toEqual({ directory: { lib: { directory: { 'a.ts': { file: { contents: 'a' } } } } } });
		const bin = (tree.static as { directory: Record<string, { file: { contents: Uint8Array } }> }).directory['x.bin'].file.contents;
		expect([...bin]).toEqual([1, 2]);
		expect(parentDir('src/lib/a.ts')).toBe('src/lib');
		expect(parentDir('a.ts')).toBe('');
	});

	it('finds what changed and what went', () => {
		const known = new Map([
			['a', '1'],
			['b', '1'],
		]);
		expect(diffManifest(known, [{ path: 'a', size_bytes: 1, updated_at: '2' }, { path: 'c', size_bytes: 1, updated_at: '1' }])).toEqual({ changed: ['a', 'c'], removed: ['b'] });
	});

	it('gives the preview its dev backend', () => {
		expect(previewEnv({ A: '1' }, null)).toEqual({ A: '1' });
		expect(previewEnv({}, { url: 'https://db', anon_key: 'k' })).toMatchObject({ PUBLIC_NOMI_URL: 'https://db', NOMI_ANON_KEY: 'k' });
		expect(previewEnv({}, { url: 'u', anon_key: 'k', service_key: 's' }).NOMI_SERVICE_KEY).toBe('s');
	});

	it('lets a quiet, finished process go, and stops one that runs too long', async () => {
		let clock = 0;
		const never = new Promise<number>(() => {});
		const hung = watchExit(never, { lastOutputAt: () => 0, settled: async () => true, quietMs: 10, timeoutMs: 1000, pollMs: 1, now: () => (clock += 5) });
		await expect(hung).resolves.toBe('hung');
		clock = 0;
		const slow = watchExit(never, { lastOutputAt: () => clock, settled: async () => false, quietMs: 10, timeoutMs: 20, pollMs: 1, now: () => (clock += 5) });
		await expect(slow).resolves.toBe('timeout');
		await expect(watchExit(Promise.resolve(0), { lastOutputAt: () => 0, settled: async () => false, quietMs: 1, timeoutMs: 1 })).resolves.toBe(0);
	});
});
