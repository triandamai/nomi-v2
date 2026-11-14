import { beforeEach, describe, expect, it, vi } from 'vitest';
import { event, json, settle } from '../../test-stubs/server';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args), apiUrl: (path: string) => `http://api.test${path}` }));

const { actions } = await import('./+page.server');

beforeEach(() => apiFetch.mockReset());

describe('showcase: remix', () => {
	it('sends someone signed out to sign in first', async () => {
		expect(await settle(() => actions.remix(event({ form: { app_id: 'a1' } })))).toMatchObject({ thrown: { status: 303, location: '/login' } });
	});

	it('copies the app and opens the copy', async () => {
		apiFetch.mockResolvedValue(json({ session_id: 's9', project_id: 'p9' }, 201));
		const signedIn = { locals: { accessToken: 't' } };
		expect(await settle(() => actions.remix(event({ form: { app_id: 'a1' }, ...signedIn })))).toMatchObject({ thrown: { status: 303, location: '/projects/session/s9' } });
		expect(apiFetch.mock.calls[0][2]).toBe('/api/showcase/a1/remix');
		apiFetch.mockResolvedValue(json({}, 409));
		expect(await settle(() => actions.remix(event({ form: { app_id: 'a1' }, ...signedIn })))).toMatchObject({ thrown: { location: '/projects' } });
		apiFetch.mockResolvedValue(json({}, 404));
		expect(await actions.remix(event({ form: { app_id: 'a1' }, ...signedIn }))).toMatchObject({ status: 404 });
		expect(await actions.remix(event({ form: {}, ...signedIn }))).toMatchObject({ status: 400 });
	});
});
