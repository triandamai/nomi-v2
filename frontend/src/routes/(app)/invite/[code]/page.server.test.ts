import { afterEach, describe, expect, it, vi } from 'vitest';
import { cookieJar, event, json, settle } from '../../../../test-stubs/server';
import { actions, load } from './+page.server';

afterEach(() => vi.unstubAllGlobals());

const invite = {
	org_id: 'org1',
	org_name: 'Studio',
	inviter: 'Ana',
	role: 'member',
	state: 'open',
	email: 'bob@example.com',
	expires_at: '2026-12-01T00:00:00Z',
};

describe('invite page', () => {
	it('loads invite data and code', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => json(invite)));
		const data = await load(event({ params: { code: 'c1' } }));
		expect(data).toMatchObject({ code: 'c1', invite });
	});

	it('returns null invite when not found (404)', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => json({}, 404)));
		const data = await load(event({ params: { code: 'c1' } }));
		expect(data).toMatchObject({ code: 'c1', invite: null });
	});

	it('accepts invite and redirects', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => json({ project_session_id: 'sess1' })));
		const thrown = await settle(() => actions.accept(event({ params: { code: 'c1' } })));
		expect(thrown).toMatchObject({ thrown: { status: 303, location: '/projects/session/sess1' } });
	});

	it('handles other_address refusal with isOther', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => new Response('this invite was sent to another address', { status: 403 })));
		const result = await actions.accept(event({ params: { code: 'c1' } }));
		expect(result).toMatchObject({
			status: 403,
			data: { isOther: true },
		});
	});
});
