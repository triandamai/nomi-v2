import { describe, expect, it, vi } from 'vitest';
import { ComposerUploads, type Sender } from './uploads.svelte';

// The composer's files and what becomes of each (UX contract audit-chat-files, G3 · G4): one
// failing never stops the others, Retry sends it again from the start, Remove takes it out.

const uploaded = (id: string, name: string) => JSON.stringify({ id, name, kind: 'pdf', mime: 'application/pdf', size: 5, status: 'ready', reference: `<attachment id="${id}"/>` });
const pdf = (name: string) => new File(['%PDF-'], name, { type: 'application/pdf' });

function controlled() {
	const calls: { name: string; resolve: (result: { status: number; body: string }) => void; progress: (fraction: number) => void; signal: AbortSignal }[] = [];
	const send: Sender = (_file, name, { onprogress, signal }) =>
		new Promise((resolve, reject) => {
			calls.push({ name, resolve, progress: onprogress, signal });
			signal.addEventListener('abort', () => reject(new DOMException('aborted', 'AbortError')));
		});
	return { calls, send };
}

const settle = () => new Promise((resolve) => setTimeout(resolve, 0));

describe('ComposerUploads', () => {
	it('uploads each file on its own: one fails, the other carries on', async () => {
		const { calls, send } = controlled();
		const announce = vi.fn();
		const uploads = new ComposerUploads({ announce }, send);
		uploads.add([pdf('bca-sep-2026.pdf'), pdf('bca-oct-2026.pdf')]);
		calls[0].progress(0.5);
		expect(uploads.chips.map((chip) => [chip.status, chip.progress])).toEqual([
			['uploading', 0.5],
			['uploading', 0],
		]);
		calls[1].resolve({ status: 0, body: '' });
		calls[0].resolve({ status: 201, body: uploaded('a', 'bca-sep-2026.pdf') });
		await settle();
		expect(uploads.chips.map((chip) => [chip.status, chip.failure])).toEqual([
			['ready', undefined],
			['failed', 'connection_lost'],
		]);
		expect(uploads.uploaded.map((file) => file.id)).toEqual(['a']);
		expect(announce).toHaveBeenCalledWith({ kind: 'failed', name: 'bca-oct-2026.pdf' });
	});

	it('Retry sends the file again from the start', async () => {
		const { calls, send } = controlled();
		const uploads = new ComposerUploads({}, send);
		uploads.add([pdf('bca-oct-2026.pdf')]);
		calls[0].resolve({ status: 507, body: 'storage is full' });
		await settle();
		expect(uploads.chips[0].failure).toBe('storage_full');
		uploads.retry(uploads.chips[0].key);
		expect(uploads.chips[0]).toMatchObject({ status: 'uploading', progress: 0, again: true });
		calls[1].resolve({ status: 201, body: uploaded('b', 'bca-oct-2026.pdf') });
		await settle();
		expect(uploads.chips[0].status).toBe('ready');
	});

	it('a file the server won’t take is rejected', async () => {
		const { calls, send } = controlled();
		const uploads = new ComposerUploads({}, send);
		uploads.add([new File(['MZ'], 'kasir-setup.exe')]);
		calls[0].resolve({ status: 415, body: 'programs can’t be attached' });
		await settle();
		expect(uploads.chips[0].status).toBe('rejected');
	});

	it('Remove stops an upload still going, and deletes one the backend holds', async () => {
		const { calls, send } = controlled();
		const discard = vi.fn();
		const uploads = new ComposerUploads({ discard }, send);
		uploads.add([pdf('a.pdf'), pdf('b.pdf')]);
		calls[1].resolve({ status: 201, body: uploaded('b', 'b.pdf') });
		await settle();
		expect(uploads.remove(uploads.chips[0].key)).toBe('a.pdf');
		expect(calls[0].signal.aborted).toBe(true);
		uploads.remove(uploads.chips[0].key);
		expect(discard).toHaveBeenCalledWith('b');
		expect(uploads.chips).toEqual([]);
	});

	it('brings back files a kept draft holds, ready', () => {
		const uploads = new ComposerUploads({}, controlled().send);
		uploads.restore([{ id: 'f1', name: 'struk.jpg', kind: 'image', mime: 'image/jpeg', size: 70, status: 'pending', reference: '' }]);
		expect(uploads.chips[0]).toMatchObject({ status: 'ready', thumb: '/attachments/f1/content' });
	});
});
