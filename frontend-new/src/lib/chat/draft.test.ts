import { describe, expect, it } from 'vitest';
import { clearDraft, loadDraft, saveDraft } from './draft';

// The composer's draft (its text and the files already uploaded) kept for a trip away from the
// chat, such as to Settings › Models & providers and back (UX contract audit-chat-files, G6 #9).

function memory(): Storage {
	const items = new Map<string, string>();
	return {
		get length() {
			return items.size;
		},
		clear: () => items.clear(),
		getItem: (key) => items.get(key) ?? null,
		key: (index) => [...items.keys()][index] ?? null,
		removeItem: (key) => void items.delete(key),
		setItem: (key, value) => void items.set(key, value),
	};
}

const file = { id: 'f1', name: 'struk-belanja-nov.jpg', kind: 'image' as const, mime: 'image/jpeg', size: 70, status: 'pending' as const, reference: '<attachment id="f1" name="struk-belanja-nov.jpg" kind="image" mime="image/jpeg" size="70"/>' };

describe('chat drafts', () => {
	it('keeps the text and the uploaded files per chat, and drops them once sent', () => {
		const storage = memory();
		saveDraft('chat-1', { text: 'What’s the total?', files: [file] }, storage);
		expect(loadDraft('chat-1', storage)).toEqual({ text: 'What’s the total?', files: [file] });
		expect(loadDraft('chat-2', storage)).toBeNull();
		clearDraft('chat-1', storage);
		expect(loadDraft('chat-1', storage)).toBeNull();
	});

	it('keeps nothing for an empty composer, and reads a broken entry as none', () => {
		const storage = memory();
		saveDraft('chat-1', { text: '  ', files: [] }, storage);
		expect(storage.length).toBe(0);
		storage.setItem('nomi:draft:chat-1', '{not json');
		expect(loadDraft('chat-1', storage)).toBeNull();
	});

	it('works without storage at all', () => {
		expect(() => saveDraft('chat-1', { text: 'x', files: [] }, undefined)).not.toThrow();
		expect(loadDraft('chat-1', undefined)).toBeNull();
	});
});
