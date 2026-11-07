import { afterEach, describe, expect, it, vi } from 'vitest';
import { event, settle } from '../../test-stubs/server';
import { actions, load } from './+page.server';

afterEach(() => vi.unstubAllGlobals());

const reply = (status: number) => vi.stubGlobal('fetch', vi.fn(async () => new Response(null, { status })));
const form = (over: Record<string, string> = {}) => event({ form: { token: 't0k', password: 'brand-new-password', confirmPassword: 'brand-new-password', ...over } });

describe('reset password: load', () => {
	it('treats a missing token or one the API says is spent as expired', async () => {
		expect(await load(event({ url: 'http://localhost/reset-password' }))).toEqual({ token: '', expired: true });
		reply(410);
		expect(await load(event({ url: 'http://localhost/reset-password?token=t0k' }))).toEqual({ token: 't0k', expired: true });
	});

	it('lets a good link, or a hiccup, through to the form', async () => {
		reply(204);
		expect(await load(event({ url: 'http://localhost/reset-password?token=t0k' }))).toEqual({ token: 't0k', expired: false });
		reply(500);
		expect(await load(event({ url: 'http://localhost/reset-password?token=t0k' }))).toMatchObject({ expired: false });
	});
});

describe('reset password: action', () => {
	it('checks the password before asking the API', async () => {
		const fetch = vi.fn();
		vi.stubGlobal('fetch', fetch);
		expect(await actions.default(form({ password: 'short', confirmPassword: 'short' }))).toMatchObject({ status: 400, data: { field: 'password' } });
		expect(await actions.default(form({ confirmPassword: 'something-else-1' }))).toMatchObject({ status: 400, data: { field: 'confirm' } });
		expect(await actions.default(event({ form: { password: 'brand-new-password', confirmPassword: 'brand-new-password' } }))).toMatchObject({ status: 410, data: { expired: true } });
		expect(fetch).not.toHaveBeenCalled();
	});

	it('sends them to log in again once the password is changed', async () => {
		reply(204);
		expect(await settle(() => actions.default(form()))).toMatchObject({ thrown: { status: 303, location: '/login?reset=1' } });
	});

	it('says so when the link was spent meanwhile, or the API refused or failed', async () => {
		reply(410);
		expect(await actions.default(form())).toMatchObject({ status: 410, data: { expired: true } });
		reply(400);
		expect(await actions.default(form())).toMatchObject({ status: 400, data: { field: 'password' } });
		reply(500);
		expect(await actions.default(form())).toMatchObject({ status: 500, data: { field: null } });
	});
});
