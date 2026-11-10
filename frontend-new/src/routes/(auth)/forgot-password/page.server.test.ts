import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { actions, load } = await import('./+page.server');

const ask = (email: string, answer: Parameters<typeof fakeBackend>[0]) => {
	const api = fakeBackend(answer);
	return { api, run: () => actions.default(fakeEvent({ url: 'http://localhost/forgot-password', form: { email }, fetch: api.fetch }) as never) };
};

describe('forgot password', () => {
	it('wants an email that looks like one', async () => {
		for (const [email, problem] of [
			['', 'email_required'],
			['ana', 'email_invalid'],
		]) {
			const { api, run } = ask(email, () => new Response(null, { status: 204 }));
			expect(await run()).toMatchObject({ status: 400, data: { email, fields: { email: problem } } });
			expect(api.calls).toEqual([]);
		}
	});

	it('says the same thing whether or not the address has an account, in the page’s language', async () => {
		const { api, run } = ask(' ana@outlook.com ', () => new Response(null, { status: 204 }));
		expect(await run()).toEqual({ sent: 'ana@outlook.com', mailApp: 'outlook' });
		expect(api.calls[0]).toMatchObject({ url: 'http://api.test/api/auth/forgot', body: { email: 'ana@outlook.com', language: 'en' } });
	});

	it('says plainly when the link could not be asked for', async () => {
		expect(await ask('ana@example.com', () => json({}, 500)).run()).toMatchObject({ status: 500, data: { alert: { kind: 'forgot_failed', status: 500 } } });
		expect(await ask('ana@example.com', () => 'unreachable').run()).toMatchObject({ status: 503, data: { alert: { kind: 'unreachable' } } });
	});

	it('starts with the address the person brought', async () => {
		expect(await load(fakeEvent({ url: 'http://localhost/forgot-password?email=ana%40example.com' }) as never)).toEqual({ email: 'ana@example.com' });
	});
});
