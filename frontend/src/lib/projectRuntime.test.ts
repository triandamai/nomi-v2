import { describe, expect, it } from 'vitest';
import { capLog, cleanOutput, diffManifest, parentDir, scriptsOf, toFileTree } from './projectRuntime';

describe('project runtime helpers', () => {
	it('nests files into folders for mounting', () => {
		expect(
			toFileTree([
				{ path: 'package.json', content: '{}' },
				{ path: 'src/routes/+page.svelte', content: '<h1>Hi</h1>' },
				{ path: 'src/app.html', content: '<html></html>' },
			]),
		).toEqual({
			'package.json': { file: { contents: '{}' } },
			src: {
				directory: {
					routes: { directory: { '+page.svelte': { file: { contents: '<h1>Hi</h1>' } } } },
					'app.html': { file: { contents: '<html></html>' } },
				},
			},
		});
	});

	it('finds new, changed and deleted files', () => {
		const known = new Map([
			['a.ts', '2026-10-09T10:00:00Z'],
			['b.ts', '2026-10-09T10:00:00Z'],
			['gone.ts', '2026-10-09T10:00:00Z'],
		]);
		const diff = diffManifest(known, [
			{ path: 'a.ts', size_bytes: 1, updated_at: '2026-10-09T10:00:00Z' },
			{ path: 'b.ts', size_bytes: 1, updated_at: '2026-10-09T10:05:00Z' },
			{ path: 'new.ts', size_bytes: 1, updated_at: '2026-10-09T10:05:00Z' },
		]);
		expect(diff).toEqual({ changed: ['b.ts', 'new.ts'], removed: ['gone.ts'] });
		expect(parentDir('src/lib/a.ts')).toBe('src/lib');
		expect(parentDir('a.ts')).toBe('');
	});

	it('strips colours and spinner redraws from terminal output', () => {
		expect(cleanOutput('\u001b[32m✓\u001b[39m built in 1s\n')).toBe('✓ built in 1s\n');
		expect(cleanOutput('⠋ installing\r⠙ installing\rdone\n')).toBe('done\n');
	});

	it('keeps the end of a long log on a line boundary', () => {
		expect(capLog('one\ntwo\nthree\n', 9)).toBe('three\n');
		expect(capLog('short', 100)).toBe('short');
	});

	it('reads npm scripts, tolerating a broken package.json', () => {
		expect(scriptsOf('{"scripts":{"check":"svelte-check"}}')).toEqual({ check: 'svelte-check' });
		expect(scriptsOf('{ not json')).toEqual({});
		expect(scriptsOf(undefined)).toEqual({});
	});
});
