import { describe, expect, it } from 'vitest';
import { cleanReleaseNoteBody, loadReleaseNotes } from './releaseNotes';

describe('cleanReleaseNoteBody', () => {
	it('removes PR links and Full Changelog while keeping what changed and who made it', () => {
		const raw = [
			"## What's Changed",
			'* fix(mobile): pin Inbox header by @triandamai in https://github.com/triandamai/nomi-v2/pull/352',
			'* feat(chat): horizontal composer by @marioprasetyo in [#355](https://github.com/triandamai/nomi-v2/pull/355)',
			'* chore: cleanup by @triandamai (#356)',
			'',
			'',
			'**Full Changelog**: https://github.com/triandamai/nomi-v2/compare/v0.1.9...v0.1.20',
		].join('\r\n');

		expect(cleanReleaseNoteBody(raw)).toBe(
			[
				"## What's Changed",
				'* fix(mobile): pin Inbox header by @triandamai',
				'* feat(chat): horizontal composer by @marioprasetyo',
				'* chore: cleanup by @triandamai',
			].join('\n'),
		);
	});
});

describe('loadReleaseNotes', () => {
	it('renders the build’s environment’s notes to sanitised HTML without PR links', async () => {
		const notes = {
			environment: 'production' as const,
			releases: [
				{
					version: '0.1.6',
					name: 'v0.1.6',
					publishedAt: '2026-11-21T02:34:59Z',
					url: 'https://github.com/triandamai/nomi-v2/releases/tag/v0.1.6',
					body: '## What’s Changed\n* fix: phones by @triandamai in https://github.com/triandamai/nomi-v2/pull/348\n* <script>alert(1)</script>safe',
				},
			],
		};
		const [release] = await loadReleaseNotes(notes, '0.1.6');
		expect(release).toMatchObject({ version: '0.1.6', url: 'https://github.com/triandamai/nomi-v2/releases/tag/v0.1.6' });
		expect(release.html).toContain('<h2>');
		expect(release.html).toContain('fix: phones by @triandamai');
		expect(release.html).not.toContain('pull/348');
		expect(release.html).not.toContain('<script');
		expect(release).not.toHaveProperty('body');
	});

	it('has nothing for another environment', async () => {
		expect(await loadReleaseNotes({ environment: 'staging', releases: [] }, '0.1.6')).toEqual([]);
	});
});
