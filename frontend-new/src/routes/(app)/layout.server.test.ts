import { describe, expect, it, vi } from 'vitest';
import { fakeEvent, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$lib/server/maintenance', () => ({ getMaintenance: async () => false }));
vi.mock('$lib/server/shell', () => ({ loadShell: async () => ({ person: { name: 'Siti' } }) }));

const { load } = await import('./+layout.server');

const visit = (url: string, locals: Partial<App.Locals> = {}) => {
	const event = fakeEvent({ url, locals }) as unknown as Record<string, unknown>;
	return load({ ...event, depends: () => {}, untrack: <T>(run: () => T) => run() } as never);
};

describe('the app’s front door', () => {
	it('shows a signed-out visitor at the bare root the landing page, as nomi.build’s old app does', async () => {
		expect(await thrown(() => visit('http://localhost/'))).toMatchObject({ status: 303, location: '/welcome' });
	});

	it('still sends anyone signed out going somewhere in particular to sign in, and back after', async () => {
		expect(await thrown(() => visit('http://localhost/?from=email'))).toMatchObject({ status: 303, location: '/sign-in?redirect_to=%2F%3Ffrom%3Demail' });
		expect(await thrown(() => visit('http://localhost/chats'))).toMatchObject({ status: 303, location: '/sign-in?redirect_to=%2Fchats' });
	});

	it('opens Home for someone signed in', async () => {
		expect(await visit('http://localhost/', { accessToken: 't' })).toMatchObject({ person: { name: 'Siti' } });
	});
});
