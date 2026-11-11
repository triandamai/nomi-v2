import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from './testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const profile = await import('./profile');

function eventWith(answer: Parameters<typeof fakeBackend>[0]) {
	const backend = fakeBackend(answer);
	return { backend, event: fakeEvent({ fetch: backend.fetch, locals: { accessToken: 'token' } }) };
}

describe('the profile on the server', () => {
	it('loads the profile and the projects that are live, to feature', async () => {
		const { backend, event } = eventWith((call) =>
			call.url.endsWith('/api/profile')
				? json({ username: 'mario' })
				: json({ org_id: 'o1', projects: [{ id: 'p1', name: 'salah-daily-tracker', shipyard_app: 'salah' }, { id: 'p2', name: 'draft', shipyard_app: null }] }),
		);
		expect(await profile.loadProfile(event)).toEqual({ profile: { username: 'mario' }, live: [{ id: 'p1', name: 'salah-daily-tracker' }] });
		expect(backend.calls.map((call) => call.url).sort()).toEqual(['http://api.test/api/profile', 'http://api.test/api/projects']);
	});

	it('still loads when the projects can’t be listed, with none to feature', async () => {
		const { event } = eventWith((call) => (call.url.endsWith('/api/profile') ? json({ username: null }) : new Response('', { status: 500 })));
		expect(await profile.loadProfile(event)).toEqual({ profile: { username: null }, live: [] });
	});

	it('says it couldn’t load when the profile fails', async () => {
		const { event } = eventWith(() => 'unreachable');
		expect(await profile.loadProfile(event)).toBeNull();
	});

	it('saves what changed and answers with the profile as it is now', async () => {
		const { backend, event } = eventWith(() => json({ username: 'mario_p' }));
		expect(await profile.saveProfile(event, { username: 'mario_p', show_crew_publicly: true })).toEqual({ ok: true, data: { username: 'mario_p' } });
		expect(backend.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/profile', body: { username: 'mario_p', show_crew_publicly: true } });
	});

	it('passes on a username that’s taken', async () => {
		const { event } = eventWith(() => new Response('that username is already taken', { status: 409 }));
		expect(await profile.saveProfile(event, { username: 'taken' })).toEqual({ ok: false, status: 409, message: 'that username is already taken' });
	});

	it('asks whether a username is free', async () => {
		const { backend, event } = eventWith(() => json({ available: false, reason: 'Username is already taken' }));
		expect(await profile.checkUsername(event, 'mario p')).toEqual({ available: false, reason: 'Username is already taken' });
		expect(backend.calls[0].url).toBe('http://api.test/api/profile/check-username?username=mario%20p');
	});
});
