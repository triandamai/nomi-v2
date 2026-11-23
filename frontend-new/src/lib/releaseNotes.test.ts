import { afterEach, describe, expect, it } from 'vitest';
import { environmentOf, markSeen, releasesFor, SEEN_KEY, shouldShow, type ReleaseNotes } from './releaseNotes';
import { releaseNotes } from '../../release-notes.js';

const release = (version: string) => ({ version, name: `v${version}`, publishedAt: '2026-11-22T00:32:09Z', url: '', body: `notes for ${version}` });

describe('environmentOf', () => {
	it('tells staging from production by the tag, as the deploy workflows do', () => {
		expect(environmentOf('0.1.7-staging4')).toBe('staging');
		expect(environmentOf('v0.1.7-staging')).toBe('staging');
		expect(environmentOf('0.1.6')).toBe('production');
		expect(environmentOf('v0.1.6')).toBe('production');
		expect(environmentOf('0.1.6-12-g419e492')).toBeNull();
		expect(environmentOf('dev')).toBeNull();
	});
});

describe('releasesFor', () => {
	const staging: ReleaseNotes = { environment: 'staging', releases: [release('0.1.7-staging5'), release('0.1.7-staging4'), release('0.1.6'), release('0.1.7-staging3')] };

	it('shows a build only its own environment’s releases, from its own on', () => {
		expect(releasesFor(staging, '0.1.7-staging4').map((r) => r.version)).toEqual(['0.1.7-staging4', '0.1.7-staging3']);
	});

	it('shows nothing when the build’s environment isn’t the notes’', () => {
		expect(releasesFor(staging, '0.1.6')).toEqual([]);
		expect(releasesFor({ environment: 'production', releases: [release('0.1.6')] }, '0.1.7-staging4')).toEqual([]);
		expect(releasesFor(staging, 'dev')).toEqual([]);
		expect(releasesFor(null, '0.1.6')).toEqual([]);
	});
});

describe('shouldShow and markSeen', () => {
	const storage = () => {
		const items = new Map<string, string>();
		return { getItem: (key: string) => items.get(key) ?? null, setItem: (key: string, value: string) => void items.set(key, value) };
	};

	it('opens once per version', () => {
		const store = storage();
		const releases = [release('0.1.7')];
		expect(shouldShow(releases, '0.1.7', store)).toBe(true);
		markSeen('0.1.7', store);
		expect(store.getItem(SEEN_KEY)).toBe('0.1.7');
		expect(shouldShow(releases, '0.1.7', store)).toBe(false);
		expect(shouldShow([release('0.1.8'), ...releases], '0.1.8', store)).toBe(true);
	});

	it('never opens for a version without notes', () => {
		expect(shouldShow([release('0.1.6')], '0.1.7', storage())).toBe(false);
	});

	it('survives blocked storage', () => {
		const blocked = { getItem: () => { throw new Error('blocked'); }, setItem: () => { throw new Error('blocked'); } };
		expect(shouldShow([release('0.1.7')], '0.1.7', blocked)).toBe(true);
		expect(() => markSeen('0.1.7', blocked)).not.toThrow();
		expect(shouldShow([release('0.1.7')], '0.1.7', undefined)).toBe(true);
	});
});

describe('release-notes.js (the build’s RELEASE_NOTES)', () => {
	const before = process.env.RELEASE_NOTES;
	afterEach(() => {
		if (before === undefined) delete process.env.RELEASE_NOTES;
		else process.env.RELEASE_NOTES = before;
	});
	const payload = { environment: 'staging', releases: [{ version: 'v0.1.7-staging4', name: 'v0.1.7-staging4', publishedAt: '2026-11-22T00:32:09Z', url: 'https://github.com/x', body: '## What’s Changed\r\n* fix\r\n' }] };

	it('reads base64 JSON, as CI passes it, and drops the v and Windows line endings', () => {
		process.env.RELEASE_NOTES = Buffer.from(JSON.stringify(payload)).toString('base64');
		expect(releaseNotes()).toEqual({ environment: 'staging', releases: [{ version: '0.1.7-staging4', name: 'v0.1.7-staging4', publishedAt: '2026-11-22T00:32:09Z', url: 'https://github.com/x', body: '## What’s Changed\n* fix' }] });
	});

	it('reads raw JSON too', () => {
		process.env.RELEASE_NOTES = JSON.stringify(payload);
		expect(releaseNotes()?.releases[0].version).toBe('0.1.7-staging4');
	});

	it('builds without notes when there are none or they can’t be read', () => {
		delete process.env.RELEASE_NOTES;
		expect(releaseNotes()).toBeNull();
		process.env.RELEASE_NOTES = 'not json';
		expect(releaseNotes()).toBeNull();
		process.env.RELEASE_NOTES = JSON.stringify({ environment: 'preview', releases: [] });
		expect(releaseNotes()).toBeNull();
	});
});
