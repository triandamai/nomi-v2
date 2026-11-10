import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { actions, load } = await import('./+page.server');

const signedIn = { locals: { accessToken: 'a1' } };

describe('first run: the page', () => {
	it('needs a sign-in, and comes back here after it', async () => {
		const event = fakeEvent({ url: 'http://localhost/first-run?redirect_to=%2Finvite%2FXk29' });
		expect(await thrown(() => load(event as never))).toMatchObject({ status: 303, location: '/sign-in?redirect_to=%2Ffirst-run%3Fredirect_to%3D%252Finvite%252FXk29' });
	});

	it('starts with the name the account has, or the address before the @', async () => {
		const named = fakeEvent({ url: 'http://localhost/first-run', fetch: fakeBackend(() => json({ display_name: 'Mario', email: 'mario@nomi.test' })).fetch, ...signedIn });
		expect(await load(named as never)).toEqual({ name: 'Mario', greeting: false });
		const unnamed = fakeEvent({ url: 'http://localhost/first-run', fetch: fakeBackend(() => json({ display_name: null, email: 'ana.s@nomi.test' })).fetch, ...signedIn });
		expect(await load(unnamed as never)).toEqual({ name: 'ana.s', greeting: false });
		const unknown = fakeEvent({ url: 'http://localhost/first-run', fetch: fakeBackend(() => json({}, 500)).fetch, ...signedIn });
		expect(await load(unknown as never)).toEqual({ name: '', greeting: false });
	});
});

describe('first run: the name', () => {
	const save = (name: string, answer: Parameters<typeof fakeBackend>[0], url = 'http://localhost/first-run') => {
		const api = fakeBackend(answer);
		return { api, run: () => actions.default(fakeEvent({ url, form: { name }, fetch: api.fetch, ...signedIn }) as never) };
	};

	it('saves only the name, then goes where the person was going', async () => {
		const { api, run } = save(' Mario ', () => json({ display_name: 'Mario' }), 'http://localhost/first-run?redirect_to=%2Finvite%2FXk29');
		expect(await thrown(run)).toMatchObject({ status: 303, location: '/invite/Xk29' });
		expect(api.calls).toEqual([{ url: 'http://api.test/api/profile', method: 'PUT', body: { display_name: 'Mario' }, headers: expect.objectContaining({ authorization: 'Bearer a1' }) }]);
	});

	it('wants a name, and says so when it could not be saved', async () => {
		const empty = save('  ', () => json({}));
		expect(await empty.run()).toMatchObject({ status: 400, data: { name: '', fields: { name: 'name_required' } } });
		expect(empty.api.calls).toEqual([]);
		expect(await save('Mario', () => json({}, 500)).run()).toMatchObject({ status: 500, data: { name: 'Mario', alert: { kind: 'firstrun_failed', status: 500 } } });
	});
});
