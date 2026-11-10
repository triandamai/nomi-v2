import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { actions, load } = await import('./+page.server');

const invite = {
	org_id: 'o1',
	org_name: 'Warung Bu Sari',
	inviter: 'Sari',
	role: 'member',
	state: 'open',
	email: 'a**@nomi.test',
	expires_at: '2026-11-17T00:00:00Z',
};
const PROJECT = '7b0c4a3e-1f2d-4c5b-9a8e-0d1c2b3a4f5e';

const open = (answer: Parameters<typeof fakeBackend>[0], signedIn = true) => {
	const api = fakeBackend(answer);
	const event = fakeEvent({ url: 'http://localhost/invite/Xk29', params: { code: 'Xk29' }, fetch: api.fetch, locals: signedIn ? { accessToken: 'a1' } : {} });
	return { api, run: () => load(event as never) };
};

describe('an invite: the page', () => {
	it('asks someone signed out to make an account or sign in first, without asking the backend', async () => {
		const { api, run } = open(() => json(invite), false);
		expect(await run()).toEqual({ code: 'Xk29', signedIn: false, invite: null, missing: false, failed: false });
		expect(api.calls).toEqual([]);
	});

	it('shows what the invite is for and where it stands', async () => {
		const { api, run } = open(() => json(invite));
		expect(await run()).toEqual({ code: 'Xk29', signedIn: true, invite: { ...invite, email: 'a••@nomi.test' }, missing: false, failed: false });
		expect(api.calls[0]).toMatchObject({ url: 'http://api.test/api/invites/Xk29', method: 'GET' });
	});

	it('knows an invite that does not exist, and one it could not load', async () => {
		expect(await open(() => json({ error: 'invite not found' }, 404)).run()).toMatchObject({ signedIn: true, invite: null, missing: true });
		expect(await open(() => json({}, 500)).run()).toMatchObject({ signedIn: true, invite: null, failed: true });
	});

	it('treats a sign-in that can no longer be renewed as signed out', async () => {
		expect(await open(() => json({}, 401)).run()).toMatchObject({ signedIn: false, invite: null });
	});
});

describe('accepting', () => {
	const accept = (answer: Parameters<typeof fakeBackend>[0], form: Record<string, string> = {}) => {
		const api = fakeBackend(answer);
		const event = fakeEvent({ url: 'http://localhost/invite/Xk29', params: { code: 'Xk29' }, form, fetch: api.fetch, locals: { accessToken: 'a1' } });
		return { api, run: () => actions.accept(event as never) };
	};

	it('joins the workspace and goes home', async () => {
		const { api, run } = accept(() => json({ org_id: 'o1' }));
		expect(await thrown(run)).toMatchObject({ status: 303, location: '/' });
		expect(api.calls[0]).toMatchObject({ url: 'http://api.test/api/invites/Xk29/accept', method: 'POST' });
	});

	it('opens the project an invite to build together was for', async () => {
		expect(await thrown(accept(() => json({ org_id: 'o1', project_session_id: 's1' }), { project: PROJECT }).run)).toMatchObject({ location: `/projects/${PROJECT}` });
		expect(await thrown(accept(() => json({ org_id: 'o1' }), { project: '../admin' }).run)).toMatchObject({ location: '/' });
	});

	it('says why an invite could not be taken', async () => {
		expect(await accept(() => new Response('this invite has expired', { status: 410 })).run()).toMatchObject({ status: 410, data: { problem: 'expired' } });
		expect(await accept(() => new Response('this invite has already been used', { status: 410 })).run()).toMatchObject({ status: 410, data: { problem: 'used' } });
		expect(await accept(() => new Response('this invite was sent to another address', { status: 403 })).run()).toMatchObject({ status: 403, data: { problem: 'other_address' } });
		expect(await accept(() => new Response('failed to accept invite', { status: 500 })).run()).toMatchObject({ status: 500, data: { problem: 'failed' } });
	});
});
