import { describe, expect, it, vi } from 'vitest';

const env: Record<string, string | undefined> = {};
vi.mock('$env/dynamic/private', () => ({ env }));

const { openPreview, sealPreview } = await import('./previewGrant');

const NOW = Date.UTC(2026, 10, 14, 9, 0, 0);
/** A token shaped like the backend's (the grant reads its `exp`, it never checks the signature). */
const jwt = (exp: number) => `x.${Buffer.from(JSON.stringify({ exp })).toString('base64url')}.y`;

describe('a preview grant', () => {
	it('opens to the person’s token, for its own project only', () => {
		const grant = sealPreview('p1', 'token-1', NOW);
		expect(grant).toMatch(/^[A-Za-z0-9_-]+$/);
		expect(grant).not.toContain('token-1');
		expect(openPreview(grant, 'p1', NOW + 60_000)).toBe('token-1');
		expect(openPreview(grant, 'p2', NOW)).toBeNull();
	});

	it('lasts an hour at most, and no longer than the token it holds', () => {
		expect(openPreview(sealPreview('p1', 'token-1', NOW), 'p1', NOW + 61 * 60_000)).toBeNull();
		const short = sealPreview('p1', jwt(NOW / 1000 + 300), NOW);
		expect(openPreview(short, 'p1', NOW + 299_000)).not.toBeNull();
		expect(openPreview(short, 'p1', NOW + 301_000)).toBeNull();
	});

	it('refuses anything changed, made up, or sealed with another key', () => {
		const grant = sealPreview('p1', 'token-1', NOW);
		const flipped = `${grant.slice(0, 20)}${grant[20] === 'A' ? 'B' : 'A'}${grant.slice(21)}`;
		expect(openPreview(flipped, 'p1', NOW)).toBeNull();
		expect(openPreview('not-a-grant', 'p1', NOW)).toBeNull();
		expect(openPreview('', 'p1', NOW)).toBeNull();
		env.PREVIEW_KEY = 'another key for another server';
		expect(openPreview(grant, 'p1', NOW)).toBeNull();
		const keyed = sealPreview('p1', 'token-1', NOW);
		expect(openPreview(keyed, 'p1', NOW)).toBe('token-1');
		env.PREVIEW_KEY = undefined;
	});
});
