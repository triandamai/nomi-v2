// Project files can be text (source) or binary (an image, a font). Text travels to the backend as
// it is; anything else goes as base64 with `encoding: 'base64'`, which the backend decodes back to
// the exact bytes.

/** A file on its way to the project: text as it is, or a binary file's bytes in base64. */
export interface FilePayload {
	path: string;
	content: string;
	encoding?: 'base64';
}

/** Bytes per `String.fromCharCode` call, so large files don't overflow the argument list. */
const CHUNK_BYTES = 0x8000;

const strictDecoder = new TextDecoder('utf-8', { fatal: true });

/** The bytes as text, or null when they're binary (not UTF-8, or holding NUL bytes). */
export function bytesAsText(bytes: Uint8Array): string | null {
	try {
		const text = strictDecoder.decode(bytes);
		return text.includes('\0') ? null : text;
	} catch {
		return null;
	}
}

export function bytesToBase64(bytes: Uint8Array): string {
	let binary = '';
	for (let i = 0; i < bytes.length; i += CHUNK_BYTES) {
		binary += String.fromCharCode(...bytes.subarray(i, i + CHUNK_BYTES));
	}
	return btoa(binary);
}

export function base64ToBytes(base64: string): Uint8Array {
	const binary = atob(base64);
	const bytes = new Uint8Array(binary.length);
	for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
	return bytes;
}

/** Extensions a browser shows as an image. */
const IMAGE_EXTENSIONS = new Set(['png', 'jpg', 'jpeg', 'gif', 'webp', 'avif', 'ico', 'bmp', 'svg']);

export function isImagePath(path: string): boolean {
	return IMAGE_EXTENSIONS.has(path.split('.').pop()?.toLowerCase() ?? '');
}

/** A file's bytes as what goes to the backend: text when it is text, base64 otherwise. */
export function toPayload(path: string, bytes: Uint8Array): FilePayload {
	const text = bytesAsText(bytes);
	return text === null ? { path, content: bytesToBase64(bytes), encoding: 'base64' } : { path, content: text };
}

/** What gets written into the preview's file system for a payload. */
export function payloadContents(file: { content: string; encoding?: string }): string | Uint8Array {
	return file.encoding === 'base64' ? base64ToBytes(file.content) : file.content;
}

/** How many bytes the file really holds. */
export function payloadSize(file: { content: string; encoding?: string }): number {
	if (file.encoding !== 'base64') return file.content.length;
	const padding = file.content.endsWith('==') ? 2 : file.content.endsWith('=') ? 1 : 0;
	return (file.content.length * 3) / 4 - padding;
}
