import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

const run = (name: keyof typeof actions, form: Record<string, string>, answer: Parameters<typeof fakeBackend>[0] = () => json({ username: 'mario' })) => {
	const api = fakeBackend(answer);
	const event = fakeEvent({ url: 'http://localhost/settings/profile', form, fetch: api.fetch, locals: { accessToken: 'token' } });
	return { api, result: actions[name](event as never) };
};

describe('Settings › Profile', () => {
	it('names itself for the top bar and loads the profile with the live projects', async () => {
		const api = fakeBackend((call) => (call.url.endsWith('/api/profile') ? json({ username: 'mario' }) : json({ projects: [{ id: 'p1', name: 'salah', shipyard_app: 'salah' }] })));
		const data = await load(fakeEvent({ fetch: api.fetch, locals: { accessToken: 'token' } }) as never);
		expect(data).toEqual({ title: 'Profile', profile: { username: 'mario' }, live: [{ id: 'p1', name: 'salah' }] });
	});

	it('shows the error page when the profile can’t load', async () => {
		const api = fakeBackend(() => 'unreachable');
		expect(await thrown(() => load(fakeEvent({ fetch: api.fetch, locals: { accessToken: 'token' } }) as never))).toMatchObject({ status: 503 });
	});

	it('saves the name, username, what’s public and the showcase in its order', async () => {
		const { api, result } = run('save', { display_name: ' Mario P ', username: '@mario_p', show_stats_publicly: 'true', show_crew_publicly: 'false', showcase: '["p2","p1"]' });
		expect(await result).toEqual({ saved: true });
		expect(api.calls[0].body).toEqual({ display_name: 'Mario P', username: 'mario_p', show_stats_publicly: true, show_crew_publicly: false, showcase_project_ids: ['p2', 'p1'] });
	});

	it('says what’s wrong with a username before asking the backend', async () => {
		const { api, result } = run('save', { display_name: 'Mario', username: 'mario-p', show_stats_publicly: 'true', show_crew_publicly: 'true', showcase: '[]' });
		expect(await result).toMatchObject({ status: 400, data: { field: 'username', message: 'Letters, numbers, underscores and periods only.' } });
		expect(api.calls).toEqual([]);
	});

	it('features three projects at most', async () => {
		const { api, result } = run('save', { display_name: 'Mario', username: 'mario', show_stats_publicly: 'true', show_crew_publicly: 'true', showcase: '["a","b","c","d"]' });
		expect(await result).toMatchObject({ status: 400 });
		expect(api.calls).toEqual([]);
	});

	it('says plainly when the username is taken', async () => {
		const { result } = run('save', { display_name: 'Mario', username: 'taken', show_stats_publicly: 'true', show_crew_publicly: 'true', showcase: '[]' }, () => new Response('that username is already taken', { status: 409 }));
		expect(await result).toMatchObject({ status: 409, data: { field: 'username', message: 'That username is taken. Try another.' } });
	});

	it('removes the photo', async () => {
		const { api, result } = run('removePhoto', {});
		expect(await result).toEqual({ photoRemoved: true });
		expect(api.calls[0].body).toEqual({ avatar_url: '' });
	});
});
