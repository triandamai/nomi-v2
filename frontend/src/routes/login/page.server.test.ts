import { afterEach, describe, expect, it, vi } from 'vitest';
import { event } from '../../test-stubs/server';
import { actions, load } from './+page.server';

afterEach(() => vi.unstubAllGlobals());

const reply = (status: number) => vi.stubGlobal('fetch', vi.fn(async () => new Response('{}', { status })));

describe('login', () => {
	it('points a failed sign-in at the password field', async () => {
		reply(401);
		expect(await actions.default(event({ form: { email: 'ana@example.com', password: 'nope' } }))).toMatchObject({ status: 401, data: { field: 'password' } });
		expect(await actions.default(event({ form: { email: '', password: 'x' } }))).toMatchObject({ status: 400, data: { field: 'email' } });
		expect(await actions.default(event({ form: { email: 'ana@example.com', password: '' } }))).toMatchObject({ status: 400, data: { field: 'password' } });
	});

	it('keeps errors that are not about a field out of the fields', async () => {
		for (const status of [429, 502, 500]) {
			reply(status);
			expect(await actions.default(event({ form: { email: 'ana@example.com', password: 'x' } }))).toMatchObject({ status, data: { field: null } });
		}
	});

	it('knows when the person is back from choosing a new password', async () => {
		reply(404);
		expect(await load(event({ url: 'http://localhost/login?reset=1' }))).toMatchObject({ passwordChanged: true });
		expect(await load(event({ url: 'http://localhost/login' }))).toMatchObject({ passwordChanged: false });
	});
});
