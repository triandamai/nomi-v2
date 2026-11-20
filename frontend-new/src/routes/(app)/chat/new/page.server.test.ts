import { describe, expect, it, vi } from 'vitest';
import { fakeEvent } from '$lib/server/testing/fakeEvent';

vi.mock('$lib/server/chat', () => ({ modelOptions: async () => ({ groups: [] }) }));

const { load } = await import('./+page.server');

describe('a new chat', () => {
	it('starts from a message carried in ?q= (the landing page’s composer), unsent and at most 2,000 characters', async () => {
		expect(await load(fakeEvent({ url: 'http://localhost/chat/new?q=Catat%20uang%20kos' }) as never)).toMatchObject({ draft: 'Catat uang kos' });
		expect(((await load(fakeEvent({ url: `http://localhost/chat/new?q=${'a'.repeat(3000)}` }) as never)) as { draft: string }).draft).toHaveLength(2000);
	});

	it('starts empty without one', async () => {
		expect(await load(fakeEvent({ url: 'http://localhost/chat/new' }) as never)).toMatchObject({ draft: '' });
	});
});
