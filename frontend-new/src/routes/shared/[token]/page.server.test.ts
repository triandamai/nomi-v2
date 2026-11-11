import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load } = await import('./+page.server');

const run = async (answer: Response, accessToken?: string) => {
	const api = fakeBackend(() => answer);
	const result = await load(fakeEvent({ params: { token: 'tok' }, fetch: api.fetch, locals: { accessToken } }) as never);
	return { api, result };
};

describe('a shared link', () => {
	it('opens read-only with its messages rendered, as the person sharing chose', async () => {
		const { api, result } = await run(
			json({ title: 'Belanja •••', scope: 'chat', masked: true, expires_at: '2026-11-18T00:00:00Z', messages: [{ sender: 'user', agent_display_name: null, content: 'Berapa?', created_at: '2026-11-11T05:00:00Z' }, { sender: 'assistant', agent_display_name: 'Dana', content: 'Rp **•••**', created_at: '2026-11-11T05:00:02Z' }] }),
		);
		expect(api.calls[0].url).toBe('http://api.test/api/shared/tok');
		expect(result).toMatchObject({ state: 'open', chat: { title: 'Belanja •••', masked: true, expiresAt: '2026-11-18T00:00:00Z' } });
		expect((result as { chat: { messages: { html: string }[] } }).chat.messages[1].html).toContain('<strong>•••</strong>');
	});

	it('says a link that isn’t there (expired, turned off or never made) isn’t available', async () => {
		expect((await run(new Response('link not found', { status: 404 }))).result).toEqual({ state: 'gone' });
	});

	it('asks someone signed out to sign in when the link is for signed-in people only', async () => {
		expect((await run(new Response('sign in to open this link', { status: 401 }))).result).toEqual({ state: 'signin' });
	});
});
