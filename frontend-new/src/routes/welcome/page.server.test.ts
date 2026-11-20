import { describe, expect, it } from 'vitest';
import { fakeEvent, thrown } from '$lib/server/testing/fakeEvent';

const { actions, load } = await import('./+page.server');

describe('the landing page', () => {
	it('is for people who aren’t signed in: the signed-in go to Home', async () => {
		expect(await load(fakeEvent({ url: 'http://localhost/welcome' }) as never)).toEqual({});
		expect(await thrown(() => load(fakeEvent({ url: 'http://localhost/welcome', locals: { accessToken: 't' } }) as never))).toMatchObject({ status: 307, location: '/' });
	});

	it('sends what was typed to sign in, then to a new chat with it in the field', async () => {
		const event = fakeEvent({ url: 'http://localhost/welcome?/start', form: { text: '  Bikinin halaman booking & bayar QRIS  ' } });
		expect(await thrown(() => actions.start(event as never))).toMatchObject({
			status: 303,
			location: `/sign-in?redirect_to=${encodeURIComponent(`/chat/new?q=${encodeURIComponent('Bikinin halaman booking & bayar QRIS')}`)}`,
		});
	});

	it('goes straight to the chat for someone already signed in, and to a blank one with nothing typed', async () => {
		const signedIn = fakeEvent({ url: 'http://localhost/welcome?/start', form: { text: 'Catat uang kos' }, locals: { accessToken: 't' } });
		expect(await thrown(() => actions.start(signedIn as never))).toMatchObject({ status: 303, location: '/chat/new?q=Catat%20uang%20kos' });
		const empty = fakeEvent({ url: 'http://localhost/welcome?/start', form: { text: '   ' } });
		expect(await thrown(() => actions.start(empty as never))).toMatchObject({ status: 303, location: `/sign-in?redirect_to=${encodeURIComponent('/chat/new')}` });
	});

	it('keeps at most 2,000 characters of it', async () => {
		const event = fakeEvent({ url: 'http://localhost/welcome?/start', form: { text: 'a'.repeat(5000) }, locals: { accessToken: 't' } });
		expect(await thrown(() => actions.start(event as never))).toMatchObject({ location: `/chat/new?q=${'a'.repeat(2000)}` });
	});
});
