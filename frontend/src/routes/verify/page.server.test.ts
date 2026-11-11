import { afterEach, describe, expect, it, vi } from 'vitest';
import { cookieJar, event, json, settle } from '../../test-stubs/server';
import { actions, load } from './+page.server';
import { setPendingSignIn } from '$lib/server/signIn';

afterEach(() => vi.unstubAllGlobals());

const pending = { challengeId: 'c1', email: 'test@example.com', isNew: false, redirectTo: '/' };

describe('verify page.server', () => {
	it('load redirects if no pending sign-in', async () => {
		const cookies = cookieJar();
		const res = await settle(() => load(event({ cookies })));
		expect(res).toHaveProperty('thrown');
	});

	it('load fetches challenge if pending sign-in exists', async () => {
		const cookies = cookieJar();
		setPendingSignIn(cookies, pending);
		const mockFetch = vi.fn(async () => json({ email: 'test@example.com', attempts_left: 3, seconds_left: 60 }));
		vi.stubGlobal('fetch', mockFetch);

		const data = await load(event({ cookies }));
		expect(data).toMatchObject({ expired: false, isNew: false, verification: { attempts_left: 3 } });
	});

	it('verify rejects invalid codes', async () => {
		const cookies = cookieJar();
		setPendingSignIn(cookies, pending);

		const res = await actions.verify(event({ cookies, form: { code: '12' } }));
		expect(res).toMatchObject({ status: 400 });
	});

	it('verify handles wrong_code failure', async () => {
		const cookies = cookieJar();
		setPendingSignIn(cookies, pending);
		const mockFetch = vi.fn(async () => json({ error: 'wrong_code', attempts_left: 2 }, 400));
		vi.stubGlobal('fetch', mockFetch);

		const res = await actions.verify(event({ cookies, form: { code: '123456' } }));
		expect(res).toMatchObject({ status: 400 });
	});

	it('verify handles too_many_attempts lock', async () => {
		const cookies = cookieJar();
		setPendingSignIn(cookies, pending);
		const mockFetch = vi.fn(async () => json({ error: 'too_many_attempts' }, 410));
		vi.stubGlobal('fetch', mockFetch);

		const res = await actions.verify(event({ cookies, form: { code: '123456' } }));
		expect(res).toMatchObject({ status: 410, data: { expired: true } });
	});

	it('verify completes sign-in on success', async () => {
		const cookies = cookieJar();
		setPendingSignIn(cookies, pending);
		const mockFetch = vi.fn(async (url: RequestInfo | URL) => {
			const u = String(url);
			if (u.includes('/api/auth/verify')) {
				return json({ access_token: 'acc', refresh_token: 'ref', expires_in: 3600 });
			}
			return json({});
		});
		vi.stubGlobal('fetch', mockFetch);

		const res = await settle(() => actions.verify(event({ cookies, form: { code: '123456' } })));
		expect(res).toHaveProperty('thrown');
	});
});
