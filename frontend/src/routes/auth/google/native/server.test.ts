import { afterEach, describe, expect, it, vi } from 'vitest';
import { cookieJar, event, json, settle } from '../../../../test-stubs/server';
import { GET } from './+server';
import { GET as callback } from '../callback/+server';
import { GOOGLE_NATIVE_COOKIE } from '$lib/server/googleNative';

afterEach(() => vi.unstubAllGlobals());

describe('Google sign-in from the native app', () => {
	it('marks the browser and goes on to Google', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => json({ url: 'https://accounts.google.com/o?x=1' })));
		const cookies = cookieJar();
		const result = (await settle(() => GET(event({ cookies })))) as { thrown: { location: string } };
		expect(result.thrown.location).toBe('https://accounts.google.com/o?x=1');
		expect(cookies.get(GOOGLE_NATIVE_COOKIE)).toBe('1');
	});

	it('sends the app back its answer when Google is unavailable', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => json({}, 503)));
		const result = (await settle(() => GET(event()))) as { thrown: { location: string } };
		expect(result.thrown.location).toBe('nomi://auth/google?error=unavailable');
	});

	it('hands Google’s code to the app instead of signing the browser in', async () => {
		const fetch = vi.fn();
		vi.stubGlobal('fetch', fetch);
		const cookies = cookieJar({ [GOOGLE_NATIVE_COOKIE]: '1' });
		const result = (await settle(() => callback(event({ cookies, url: 'http://localhost/auth/google/callback?code=c0de&state=st' })))) as {
			thrown: { location: string };
		};
		expect(result.thrown.location).toBe('nomi://auth/google?code=c0de&state=st');
		expect(cookies.get(GOOGLE_NATIVE_COOKIE)).toBeUndefined();
		expect(fetch).not.toHaveBeenCalled();
	});
});
