import { describe, expect, it } from 'vitest';
import { appendOutput, capLog, cleanOutput, diffManifest, parentDir, previewEnv, scriptsOf, toFileTree } from './projectRuntime';

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

describe('appendOutput', () => {
	it('drops the blank screenful Vite prints before clearing the screen', () => {
		const vite = `${'\n'.repeat(40)}\u001b[1;1H\u001b[0J  VITE v8 ready\n`;
		expect(appendOutput('200 GET /\n', vite)).toBe('200 GET /\n  VITE v8 ready\n');
		expect(appendOutput('', `\n\n\n\u001b[2J\u001b[3J\u001b[Hready\n`)).toBe('ready\n');
	});

	it('collapses long runs of blank lines but keeps single gaps', () => {
		expect(appendOutput('a\n', '\n\n\n\nb\n')).toBe('a\n\nb\n');
		expect(appendOutput('a\n', '\nb\n')).toBe('a\n\nb\n');
	});

	it('gives the preview its dev backend, and the service key only when Nomi sent one', () => {
		const base = { NOMI_PREVIEW: '1' };
		const member = previewEnv(base, { url: 'https://todo-dev.api.relay.test', anon_key: 'anon' });
		expect(member).toEqual({
			NOMI_PREVIEW: '1',
			PUBLIC_NOMI_URL: 'https://todo-dev.api.relay.test',
			PUBLIC_NOMI_KEY: 'anon',
			NOMI_URL: 'https://todo-dev.api.relay.test',
			NOMI_ANON_KEY: 'anon',
		});
		expect(previewEnv(base, { url: 'u', anon_key: 'a', service_key: 's' }).NOMI_SERVICE_KEY).toBe('s');
		expect(previewEnv(base, null)).toEqual(base);
	});
});
