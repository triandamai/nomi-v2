import { m } from '$lib/paraglide/messages';
// Files attached in the composer are uploaded as soon as they're picked (uploadFile, through
// /attachments), and the message carries one reference tag per file after the typed text:
// <attachment id="…" name="…" kind="…" mime="…" size="…"/>. The backend reads each file (text
// of documents, a description of an image, a transcript of audio) and routes the message to
// the Files agent. The chat shows the tags as previews and cards (see splitAttachments).
// Messages from before uploads carried a text file inline instead, as
// <attachment name="…" kind="file|voice">…</attachment>; those still display.

export type AttachmentKind =
	| 'text'
	| 'pdf'
	| 'document'
	| 'spreadsheet'
	| 'presentation'
	| 'image'
	| 'audio'
	| 'voice'
	| 'video'
	| 'other'
	/** An older message's inline text file. */
	| 'file';

/** A file in a sent message. */
export interface Attachment {
	name: string;
	kind: AttachmentKind;
	/** Uploaded files have an id (and their type and size); older inline ones carry their text. */
	id?: string;
	mime?: string;
	size?: number;
	text?: string;
}

/** What the backend says about an upload. */
export interface UploadedFile {
	id: string;
	name: string;
	kind: AttachmentKind;
	mime: string;
	size: number;
	status: 'pending' | 'processing' | 'ready' | 'failed';
	/** The tag a message carries to point at this file. */
	reference: string;
}

const MB = 1024 * 1024;
export const MAX_ATTACHMENTS = 10;
export const MAX_FILE_BYTES = 10 * MB;
export const MAX_VIDEO_BYTES = 50 * MB;
export const MAX_TOTAL_BYTES = 100 * MB;

export function isVideoFile(name: string, type: string): boolean {
	return type.startsWith('video/') || /\.(mp4|mov|m4v|webm|mkv|avi|3gp)$/i.test(name);
}

/** The most a file may be: videos get more room. */
export function maxBytesFor(name: string, type: string): number {
	return isVideoFile(name, type) ? MAX_VIDEO_BYTES : MAX_FILE_BYTES;
}

/** Why `file` can't join `alreadyAttached`, or null when it can. */
export function refusal(file: { name: string; size: number; type: string }, alreadyAttached: { size: number }[]): string | null {
	if (alreadyAttached.length >= MAX_ATTACHMENTS) return m.chat_max_files({ count: MAX_ATTACHMENTS });
	if (file.size === 0) return m.chat_file_empty({ name: file.name });
	const max = maxBytesFor(file.name, file.type);
	if (file.size > max) {
		return isVideoFile(file.name, file.type)
			? m.chat_video_too_big({ name: file.name, max: formatBytes(max) })
			: m.chat_file_too_big({ name: file.name, max: formatBytes(max) });
	}
	const total = alreadyAttached.reduce((sum, f) => sum + f.size, 0);
	if (total + file.size > MAX_TOTAL_BYTES) return m.chat_total_too_big({ max: formatBytes(MAX_TOTAL_BYTES) });
	return null;
}

/** "Voice note (0:42)" */
export function voiceNoteName(seconds: number): string {
	const s = Math.max(0, Math.round(seconds));
	return m.voice_note_name({ length: `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}` });
}

/** The typed text followed by each file's reference tag. */
export function composeMessage(text: string, references: string[]): string {
	return [text.trim(), ...references].filter(Boolean).join('\n\n');
}

const REFERENCE = /<attachment ([^<>]*?)\/>/g;
const INLINE = /<attachment name="([^"]*)"(?: kind="(file|voice)")?>\n([\s\S]*?)\n<\/attachment>/g;

function attribute(attributes: string, name: string): string | undefined {
	return new RegExp(`(?:^|\\s)${name}="([^"]*)"`).exec(attributes)?.[1];
}

/** The typed text and the attached files of a message. */
export function splitAttachments(content: string): { text: string; files: Attachment[] } {
	const found: { at: number; file: Attachment }[] = [];
	let text = content.replace(REFERENCE, (whole, attributes: string, at: number) => {
		const id = attribute(attributes, 'id');
		if (!id) return whole;
		const size = Number(attribute(attributes, 'size'));
		found.push({
			at,
			file: {
				id,
				name: attribute(attributes, 'name') ?? 'file',
				kind: (attribute(attributes, 'kind') as AttachmentKind | undefined) ?? 'other',
				mime: attribute(attributes, 'mime'),
				size: Number.isFinite(size) ? size : undefined,
			},
		});
		return '';
	});
	text = text.replace(INLINE, (_, name: string, kind: 'file' | 'voice' | undefined, body: string, at: number) => {
		found.push({ at, file: { name, text: body, kind: kind ?? 'file' } });
		return '';
	});
	return { text: text.trim(), files: found.sort((a, b) => a.at - b.at).map((f) => f.file) };
}

/** Where the chat loads a file, and an image's small preview. */
export function contentUrl(id: string): string {
	return `/attachments/${id}/content`;
}
export function previewUrl(id: string): string {
	return `/attachments/${id}/preview`;
}

export function formatBytes(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`;
	if (bytes < MB) return `${(bytes / 1024).toFixed(bytes < 10 * 1024 ? 1 : 0)} KB`;
	return `${(bytes / MB).toFixed(bytes < 10 * MB ? 1 : 0)} MB`;
}

export class UploadError extends Error {
	constructor(
		message: string,
		readonly status: number,
	) {
		super(message);
	}
}

/** What went wrong with an upload, for the person. */
export function uploadErrorMessage(name: string, status: number): string {
	switch (status) {
		case 413:
			return m.chat_file_too_big({ name, max: formatBytes(MAX_FILE_BYTES) });
		case 415:
			return m.chat_file_refused({ name });
		case 507:
			return m.chat_storage_full();
		case 0:
			return m.chat_upload_offline({ name });
		default:
			return m.chat_upload_failed({ name });
	}
}

export interface UploadOptions {
	/** A voice note recorded in the composer. */
	voice?: boolean;
	/** The browser's own transcript of a voice note, kept as a fallback. */
	transcript?: string;
	onprogress?: (fraction: number) => void;
	signal?: AbortSignal;
}

function send(file: Blob, name: string, options: UploadOptions): Promise<{ status: number; body: string }> {
	return new Promise((resolve, reject) => {
		const xhr = new XMLHttpRequest();
		xhr.open('POST', '/attachments');
		xhr.setRequestHeader('Content-Type', file.type || 'application/octet-stream');
		xhr.setRequestHeader('X-File-Name', encodeURIComponent(name));
		if (options.voice) xhr.setRequestHeader('X-Attachment-Kind', 'voice');
		if (options.transcript) xhr.setRequestHeader('X-Transcript', encodeURIComponent(options.transcript.slice(0, 4000)));
		xhr.upload.onprogress = (event) => {
			if (event.lengthComputable) options.onprogress?.(event.loaded / event.total);
		};
		xhr.onload = () => resolve({ status: xhr.status, body: xhr.responseText });
		xhr.onerror = () => resolve({ status: 0, body: '' });
		xhr.onabort = () => reject(new DOMException('aborted', 'AbortError'));
		options.signal?.addEventListener('abort', () => xhr.abort(), { once: true });
		xhr.send(file);
	});
}

/** Uploads one file. A signed-out session is refreshed once and the upload retried. */
export async function uploadFile(file: Blob, name: string, options: UploadOptions = {}): Promise<UploadedFile> {
	let response = await send(file, name, options);
	if (response.status === 401) {
		const refreshed = await fetch('/attachments/session', { method: 'POST' }).catch(() => null);
		if (refreshed?.ok) response = await send(file, name, options);
	}
	if (response.status === 401) {
		window.location.href = '/login';
	}
	if (response.status !== 201) throw new UploadError(uploadErrorMessage(name, response.status), response.status);
	return JSON.parse(response.body) as UploadedFile;
}

/** Removes a file taken out of the composer before it was sent. */
export async function discardUpload(id: string): Promise<void> {
	await fetch(`/attachments/${id}`, { method: 'DELETE' }).catch(() => undefined);
}
