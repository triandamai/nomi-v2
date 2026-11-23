import { describe, expect, it } from 'vitest';
import { loadReleaseNotes } from './releaseNotes';

describe('loadReleaseNotes', () => {
	it('renders the build’s environment’s notes to sanitised HTML', async () => {
		const notes = { environment: 'production' as const, releases: [{ version: '0.1.6', name: 'v0.1.6', publishedAt: '2026-11-21T02:34:59Z', url: 'https://github.com/triandamai/nomi-v2/releases/tag/v0.1.6', body: '## What’s Changed\n* fix: phones <script>alert(1)</script>' }] };
		const [release] = await loadReleaseNotes(notes, '0.1.6');
		expect(release).toMatchObject({ version: '0.1.6', url: 'https://github.com/triandamai/nomi-v2/releases/tag/v0.1.6' });
		expect(release.html).toContain('<h2>');
		expect(release.html).not.toContain('<script');
		expect(release).not.toHaveProperty('body');
	});

	it('has nothing for another environment', async () => {
		expect(await loadReleaseNotes({ environment: 'staging', releases: [] }, '0.1.6')).toEqual([]);
	});
});
