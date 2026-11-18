import type { UploadedFile } from './attachments';

// What's in a chat's composer, kept in the tab's session storage while the person steps away (to
// Settings › Models & providers, to add a model that can see a picture, and back): the text and
// the files already uploaded, which the backend still holds. Nothing is kept for an empty composer,
// and a draft is dropped once it's sent. Storage that's missing or refuses is no draft at all.

export type Draft = { text: string; files: UploadedFile[] };

const key = (chatId: string) => `nomi:draft:${chatId}`;

function storageOf(storage: Storage | undefined): Storage | undefined {
	if (storage) return storage;
	try {
		return typeof sessionStorage === 'undefined' ? undefined : sessionStorage;
	} catch {
		return undefined;
	}
}

export function saveDraft(chatId: string, draft: Draft, storage?: Storage): void {
	const store = storageOf(storage);
	try {
		if (!draft.text.trim() && draft.files.length === 0) store?.removeItem(key(chatId));
		else store?.setItem(key(chatId), JSON.stringify(draft));
	} catch {
		// Full or blocked: the draft only lives on the page.
	}
}

export function loadDraft(chatId: string, storage?: Storage): Draft | null {
	try {
		const raw = storageOf(storage)?.getItem(key(chatId));
		if (!raw) return null;
		const draft = JSON.parse(raw) as Draft;
		return typeof draft.text === 'string' && Array.isArray(draft.files) ? draft : null;
	} catch {
		return null;
	}
}

export function clearDraft(chatId: string, storage?: Storage): void {
	try {
		storageOf(storage)?.removeItem(key(chatId));
	} catch {
		// Nothing to clear.
	}
}
