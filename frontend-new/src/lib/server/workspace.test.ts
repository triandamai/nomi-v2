import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from './testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const workspace = await import('./workspace');
const eventWith = (answer: Parameters<typeof fakeBackend>[0]) => {
	const backend = fakeBackend(answer);
	return { backend, event: fakeEvent({ fetch: backend.fetch, locals: { accessToken: 'token' } }) };
};

describe('a project’s workspace on the server', () => {
	it('loads the project and its live address from the list', async () => {
		const { event } = eventWith((call) =>
			call.url.endsWith('/api/projects') ? json({ org_id: 'w', projects: [{ id: 'p1', deploy_url: 'https://warung.trian.space', deploys_count: 2 }], base_domain: 'trian.space' }) : json({ id: 'p1', name: 'warung', session_id: 's1' }),
		);
		expect(await workspace.loadProject(event, 'p1')).toEqual({ project: { id: 'p1', name: 'warung', session_id: 's1' }, live: { url: 'https://warung.trian.space', deploys: 2 }, domain: 'trian.space' });
	});

	it('says when the project isn’t there, or isn’t theirs', async () => {
		const { event } = eventWith(() => new Response('project not found', { status: 404 }));
		expect(await workspace.loadProject(event, 'p1')).toEqual({ project: null, status: 404 });
	});

	it('finds a project by its chat', async () => {
		const { backend, event } = eventWith(() => json({ id: 'p1' }));
		expect(await workspace.projectOfChat(event, 's 1')).toBe('p1');
		expect(backend.calls[0].url).toBe('http://api.test/api/projects/by-session/s%201');
		expect(await workspace.projectOfChat(eventWith(() => new Response('', { status: 404 })).event, 's1')).toBeNull();
	});
});
