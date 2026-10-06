import { describe, expect, it } from 'vitest';
import { pageTitle } from './pageTitle';

describe('pageTitle', () => {
	it('names pages by route, and Home is just Nomi', () => {
		expect(pageTitle('/(app)/money')).toBe('Money · Nomi');
		expect(pageTitle('/(app)')).toBe('Nomi');
		expect(pageTitle('/admin/(protected)/users')).toBe('Admin · Nomi');
		expect(pageTitle(null)).toBe('Nomi');
	});

	it("prefers the page's own title", () => {
		expect(pageTitle('/(app)/chat/[sessionId]', 'Bogor day trip')).toBe('Bogor day trip · Nomi');
		expect(pageTitle('/(app)/chat/[sessionId]', null)).toBe('Chat · Nomi');
		expect(pageTitle('/(app)/chat/[sessionId]', '  ')).toBe('Chat · Nomi');
	});
});
