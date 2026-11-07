import { afterEach, describe, expect, it, vi } from 'vitest';
import { event } from '../../test-stubs/server';
import { actions } from './+page.server';

afterEach(() => vi.unstubAllGlobals());

const form = (over: Record<string, string> = {}) =>
	event({ form: { email: 'ana@example.com', password: 'correct-password', confirmPassword: 'correct-password', orgName: 'Acme', ...over } });

describe('register', () => {
	it('asks for the password twice, and says which field is wrong', async () => {
		const fetch = vi.fn();
		vi.stubGlobal('fetch', fetch);
		expect(await actions.default(form({ confirmPassword: 'something-else' }))).toMatchObject({ status: 400, data: { field: 'confirm' } });
		expect(await actions.default(form({ confirmPassword: '' }))).toMatchObject({ status: 400, data: { field: 'confirm' } });
		expect(await actions.default(form({ email: '' }))).toMatchObject({ status: 400, data: { field: 'email' } });
		expect(await actions.default(form({ orgName: '' }))).toMatchObject({ status: 400, data: { field: 'orgName' } });
		expect(fetch).not.toHaveBeenCalled();
	});

	it('points a taken email at the email field', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => new Response('{}', { status: 409 })));
		expect(await actions.default(form())).toMatchObject({ status: 409, data: { field: 'email' } });
	});

	it('keeps other failures out of the fields', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => new Response('{}', { status: 500 })));
		expect(await actions.default(form())).toMatchObject({ status: 500, data: { field: null } });
	});
});
