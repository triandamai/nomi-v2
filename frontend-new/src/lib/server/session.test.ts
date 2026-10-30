import { describe, expect, it } from 'vitest';
import { needsRefresh, tokenExpiry } from './session';

/** A JWT with this payload (unsigned: only the expiry is read, never trusted for anything else). */
function jwt(payload: object): string {
	const part = (value: object) => Buffer.from(JSON.stringify(value)).toString('base64url');
	return `${part({ alg: 'HS256' })}.${part(payload)}.signature`;
}

const now = Date.UTC(2026, 9, 28, 12, 0, 0);
const inSeconds = (seconds: number) => Math.floor(now / 1000) + seconds;

describe('session tokens', () => {
	it('reads when an access token expires', () => {
		expect(tokenExpiry(jwt({ exp: inSeconds(1800) }))).toBe(now + 1_800_000);
		expect(tokenExpiry('not-a-jwt')).toBeNull();
		expect(tokenExpiry(jwt({ sub: 'no expiry' }))).toBeNull();
	});

	it('refreshes ahead of expiry, so a page load never starts with a token about to lapse', () => {
		expect(needsRefresh(undefined, now)).toBe(true);
		expect(needsRefresh(jwt({ exp: inSeconds(30) }), now)).toBe(true);
		expect(needsRefresh(jwt({ exp: inSeconds(-5) }), now)).toBe(true);
		expect(needsRefresh(jwt({ exp: inSeconds(600) }), now)).toBe(false);
		// Not a token we can read: use it, and let the API's 401 decide.
		expect(needsRefresh('opaque-token', now)).toBe(false);
	});
});
