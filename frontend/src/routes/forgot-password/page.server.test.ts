import { afterEach, describe, expect, it, vi } from 'vitest';
import { event } from '../../test-stubs/server';
import { actions } from './+page.server';

const status = (result: unknown) => (result as { status?: number }).status;

afterEach(() => vi.unstubAllGlobals());

function backend(response: Response | null) {
	const fetch = vi.fn(async () => {
		if (!response) throw new Error('offline');
		return response;
	});
	vi.stubGlobal('fetch', fetch);
	return fetch;
}

describe('forgot password', () => {
	it('asks for an email first, pointing at the field', async () => {
		const fetch = backend(new Response(null, { status: 204 }));
		expect(await actions.default(event({ form: { email: '  ' } }))).toMatchObject({ status: 400, data: { field: 'email' } });
		expect(fetch).not.toHaveBeenCalled();
	});

	it('asks the API for a link and says it was sent, trimmed', async () => {
		const fetch = backend(new Response(null, { status: 204 }));
		expect(await actions.default(event({ form: { email: ' ana@example.com ' } }))).toEqual({ sent: 'ana@example.com' });
		const [url, init] = fetch.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toContain('/api/auth/forgot');
		expect(JSON.parse(init.body as string)).toMatchObject({ email: 'ana@example.com' });
	});

	it('reports a failure instead of saying it was sent', async () => {
		backend(new Response(null, { status: 500 }));
		expect(status(await actions.default(event({ form: { email: 'ana@example.com' } })))).toBe(500);
		backend(null);
		expect(status(await actions.default(event({ form: { email: 'ana@example.com' } })))).toBe(502);
	});
});
