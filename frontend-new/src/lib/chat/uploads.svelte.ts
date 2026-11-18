import { uploadFailure, type ChipState, type UploadedFile } from './attachments';

// The composer's files, each uploaded on its own the moment it's picked (UX contract
// audit-chat-files, the attachment chip's state machine): uploading → ready on a 201, or failed
// (why: the connection, the server, storage) or rejected (the server won't take that kind of file).
// One file failing never stops the others. Retry sends the file again from the start: the backend
// can't resume an upload (M1). Remove takes a file out: an upload still going is stopped, one the
// backend holds is deleted there (it was never sent), a failed one is simply dropped.

export type Sender = (file: Blob, name: string, options: { onprogress: (fraction: number) => void; signal: AbortSignal }) => Promise<{ status: number; body: string }>;

export type Chip = ChipState & { file?: File; thumb?: string };

type Hooks = {
	/** Something to say to a screen reader; assertive for a failure. */
	announce?: (text: { kind: 'uploaded' | 'failed' | 'rejected'; name: string }) => void;
	/** Deletes an upload the backend holds (DELETE /attachments/:id). */
	discard?: (id: string) => void;
};

/** Uploads one file through /attachments (streamed to the backend), reporting real bytes sent. */
export const sendFile: Sender = (file, name, { onprogress, signal }) =>
	new Promise((resolve, reject) => {
		const xhr = new XMLHttpRequest();
		xhr.open('POST', '/attachments');
		xhr.setRequestHeader('Content-Type', file.type || 'application/octet-stream');
		xhr.setRequestHeader('X-File-Name', encodeURIComponent(name));
		xhr.upload.onprogress = (event) => {
			if (event.lengthComputable) onprogress(event.loaded / event.total);
		};
		xhr.onload = () => resolve({ status: xhr.status, body: xhr.responseText });
		xhr.onerror = () => resolve({ status: 0, body: '' });
		xhr.onabort = () => reject(new DOMException('aborted', 'AbortError'));
		signal.addEventListener('abort', () => xhr.abort(), { once: true });
		xhr.send(file);
	});

/** An upload refused as signed out: renew the sign-in once (the page's own endpoint), then again. */
const withRenewal =
	(send: Sender): Sender =>
	async (file, name, options) => {
		const first = await send(file, name, options);
		if (first.status !== 401) return first;
		const renewed = await fetch('/attachments/session', { method: 'POST' }).catch(() => null);
		return renewed?.ok ? send(file, name, options) : first;
	};

let nextKey = 1;

export class ComposerUploads {
	chips = $state<Chip[]>([]);
	#controllers = new Map<string, AbortController>();
	#send: Sender;
	#hooks: Hooks;

	constructor(hooks: Hooks = {}, send: Sender = withRenewal(sendFile)) {
		this.#hooks = hooks;
		this.#send = send;
	}

	/** Files picked: each starts uploading now. */
	add(files: Iterable<File>) {
		for (const file of files) {
			const chip: Chip = { key: `file-${nextKey++}`, name: file.name, size: file.size, status: 'uploading', progress: 0, file, thumb: imageThumb(file) };
			this.chips = [...this.chips, chip];
			void this.#upload(chip.key);
		}
	}

	/** Files the backend already holds, back from a kept draft. */
	restore(files: UploadedFile[]) {
		this.chips = [
			...this.chips,
			...files.map((uploaded): Chip => ({ key: `file-${nextKey++}`, name: uploaded.name, size: uploaded.size, status: 'ready', uploaded, thumb: uploaded.kind === 'image' ? `/attachments/${uploaded.id}/content` : undefined })),
		];
	}

	retry(key: string) {
		this.#set(key, { status: 'uploading', progress: 0, failure: undefined, again: true });
		void this.#upload(key);
	}

	/** Takes a file out of the message; the name of what's gone, for the composer's line. */
	remove(key: string): string | null {
		const chip = this.chips.find((one) => one.key === key);
		if (!chip) return null;
		this.#controllers.get(key)?.abort();
		this.#controllers.delete(key);
		if (chip.uploaded) this.#hooks.discard?.(chip.uploaded.id);
		if (chip.thumb?.startsWith('blob:')) URL.revokeObjectURL(chip.thumb);
		this.chips = this.chips.filter((one) => one.key !== key);
		return chip.name;
	}

	/** The files that made it, as they'll go with the message. */
	get uploaded(): UploadedFile[] {
		return this.chips.flatMap((chip) => (chip.status === 'ready' && chip.uploaded ? [chip.uploaded] : []));
	}

	/** Sent: the files belong to the message now, so nothing is deleted. */
	clear() {
		for (const chip of this.chips) if (chip.thumb?.startsWith('blob:')) URL.revokeObjectURL(chip.thumb);
		this.chips = [];
	}

	#set(key: string, change: Partial<Chip>) {
		this.chips = this.chips.map((chip) => (chip.key === key ? { ...chip, ...change } : chip));
	}

	async #upload(key: string) {
		const chip = this.chips.find((one) => one.key === key);
		if (!chip?.file) return;
		const controller = new AbortController();
		this.#controllers.set(key, controller);
		let result: { status: number; body: string };
		try {
			result = await this.#send(chip.file, chip.name, { onprogress: (progress) => this.#set(key, { progress }), signal: controller.signal });
		} catch {
			return; // Removed while it went.
		} finally {
			this.#controllers.delete(key);
		}
		if (!this.chips.some((one) => one.key === key)) return;
		if (result.status === 201) {
			const uploaded = JSON.parse(result.body) as UploadedFile;
			this.#set(key, { status: 'ready', progress: undefined, again: false, uploaded });
			this.#hooks.announce?.({ kind: 'uploaded', name: chip.name });
			return;
		}
		const failure = uploadFailure(result.status);
		if (failure === 'rejected') {
			this.#set(key, { status: 'rejected', progress: undefined, failure });
			this.#hooks.announce?.({ kind: 'rejected', name: chip.name });
		} else {
			this.#set(key, { status: 'failed', progress: undefined, failure });
			this.#hooks.announce?.({ kind: 'failed', name: chip.name });
		}
	}
}

/** A picture's own preview while it's in the composer (HEIC isn't one a browser draws). */
function imageThumb(file: File): string | undefined {
	if (!file.type.startsWith('image/') || /hei[cf]/i.test(file.type)) return undefined;
	try {
		return URL.createObjectURL(file);
	} catch {
		return undefined;
	}
}
