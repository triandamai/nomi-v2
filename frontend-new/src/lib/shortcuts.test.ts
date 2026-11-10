import { describe, expect, it } from 'vitest';
import { shortcutFor } from './shortcuts';

const key = (over: Partial<Parameters<typeof shortcutFor>[0]>) => ({ key: '', code: '', metaKey: false, ctrlKey: false, shiftKey: false, altKey: false, typing: false, ...over });

describe('shortcutFor', () => {
	it('knows the app shell’s keys, with ⌘ on a Mac and Ctrl elsewhere', () => {
		expect(shortcutFor(key({ key: 'k', code: 'KeyK', metaKey: true }))).toBe('new-chat');
		expect(shortcutFor(key({ key: 'K', code: 'KeyK', ctrlKey: true }))).toBe('new-chat');
		expect(shortcutFor(key({ key: ',', code: 'Comma', metaKey: true }))).toBe('settings');
		// ⇧ turns the comma into < on most layouts: the key's place is what counts.
		expect(shortcutFor(key({ key: '<', code: 'Comma', metaKey: true, shiftKey: true }))).toBe('workspace-settings');
		expect(shortcutFor(key({ key: '?', code: 'Slash', shiftKey: true }))).toBe('shortcuts');
	});

	it('leaves ? to the field someone is typing in, and plain letters alone', () => {
		expect(shortcutFor(key({ key: '?', code: 'Slash', shiftKey: true, typing: true }))).toBeNull();
		expect(shortcutFor(key({ key: 'k', code: 'KeyK' }))).toBeNull();
		expect(shortcutFor(key({ key: 'k', code: 'KeyK', metaKey: true, altKey: true }))).toBeNull();
	});

	it('still opens a new chat from inside a field, as ⌘K does everywhere', () => {
		expect(shortcutFor(key({ key: 'k', code: 'KeyK', metaKey: true, typing: true }))).toBe('new-chat');
	});
});
