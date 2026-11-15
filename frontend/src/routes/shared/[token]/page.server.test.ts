import { afterEach, describe, expect, it, vi } from 'vitest';
import { cookieJar, event, json } from '../../../test-stubs/server';
import { actions, load } from './+page.server';

afterEach(() => vi.unstubAllGlobals());

const call = (fetchFn: ReturnType<typeof vi.fn>) => fetchFn.mock.calls[0] as unknown as [string, RequestInit];

describe('shared page', () => {
	it('loads the link with the owner’s banner facts', async () => {
		vi.stubGlobal(
			'fetch',
			vi.fn(async () =>
				json({ title: 'Trip', scope: 'chat', masked: true, expires_at: null, messages: [], shared_by: 'M.', shared_at: '2026-10-22T00:00:00Z', own: true, share_id: 's', session_id: 'c', views: 3 }),
			),
		);
		const data = (await load(event({ params: { token: 'abc' }, cookies: cookieJar({ access_token: 't' }) }))) as { state: string; chat: { views: number; shared_by: string } };
		expect(data.state).toBe('ready');
		expect(data.chat).toMatchObject({ views: 3, shared_by: 'M.' });
	});

	it('stops sharing the owner’s link', async () => {
		const fetchFn = vi.fn(async () => new Response(null, { status: 204 }));
		vi.stubGlobal('fetch', fetchFn);
		const result = await actions.stop(event({ cookies: cookieJar({ access_token: 't' }), form: { session_id: 'c', share_id: 's' } }));
		expect(result).toEqual({ stopped: true });
		const [url, init] = call(fetchFn);
		expect(url).toMatch(/\/api\/sessions\/c\/shares\/s$/);
		expect(init.method).toBe('DELETE');
	});

	it('reports the page without a sign-in', async () => {
		const fetchFn = vi.fn(async () => new Response(null, { status: 201 }));
		vi.stubGlobal('fetch', fetchFn);
		const result = await actions.report(event({ params: { token: 'abc' }, form: { reason: 'spam', note: '  ads  ' } }));
		expect(result).toEqual({ reported: true });
		const [url, init] = call(fetchFn);
		expect(url).toMatch(/\/api\/shared\/abc\/report$/);
		expect(JSON.parse(String(init.body))).toEqual({ reason: 'spam', note: 'ads' });
		expect((init.headers as Record<string, string>).Authorization).toBeUndefined();
	});

	it('asks for a reason, and says when there were too many reports', async () => {
		expect(await actions.report(event({ params: { token: 'abc' }, form: { reason: 'boring' } }))).toMatchObject({ status: 400 });
		vi.stubGlobal('fetch', vi.fn(async () => new Response(null, { status: 429 })));
		expect(await actions.report(event({ params: { token: 'abc' }, form: { reason: 'other' } }))).toMatchObject({ status: 429, data: { reportError: expect.any(String) } });
	});
});
