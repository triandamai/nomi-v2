// Files attached in the composer travel inside the message text, so every model provider can
// read them: each becomes an <attachment name="…" kind="…">…</attachment> section after the typed
// text (the backend routes such messages to the Files agent). `kind` is "file", or "voice" for a
// voice note's transcript. The chat shows those sections as chips (see splitAttachments).

export type AttachmentKind = 'file' | 'voice';

export interface Attachment {
	name: string;
	text: string;
	/** Defaults to "file" (messages sent before voice notes carry no kind). */
	kind?: AttachmentKind;
}

export const MAX_ATTACHMENTS = 5;
export const MAX_FILE_BYTES = 100 * 1024;
export const MAX_TOTAL_BYTES = 200 * 1024;

const TEXT_EXTENSIONS = new Set([
	'txt', 'md', 'markdown', 'csv', 'tsv', 'json', 'yaml', 'yml', 'toml', 'xml', 'html', 'css', 'js', 'ts', 'jsx', 'tsx',
	'svelte', 'vue', 'py', 'rb', 'go', 'rs', 'java', 'kt', 'swift', 'c', 'h', 'cpp', 'cs', 'php', 'sh', 'sql', 'log', 'ini', 'env',
]);

/** Audio files can't be transcribed server-side yet; the composer suggests a voice note instead. */
export function isAudioFile(name: string, type: string): boolean {
	return type.startsWith('audio/') || /\.(mp3|m4a|wav|ogg|opus|aac|flac|webm)$/i.test(name);
}

/** "Voice note (0:42)" */
export function voiceNoteName(seconds: number): string {
	const s = Math.max(0, Math.round(seconds));
	return `Voice note (${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')})`;
}

/** Whether a file can be attached as text. Images, PDFs and other binaries can't (yet). */
export function isTextFile(name: string, type: string): boolean {
	if (type.startsWith('text/') || type === 'application/json' || type === 'application/xml') return true;
	const extension = name.split('.').pop()?.toLowerCase() ?? '';
	return TEXT_EXTENSIONS.has(extension);
}

function escapeName(name: string): string {
	return name.replace(/["<>\n]/g, '_');
}

export function composeMessage(text: string, attachments: Attachment[]): string {
	const parts = [text.trim()];
	for (const file of attachments) {
		const kind = file.kind ?? 'file';
		parts.push(`<attachment name="${escapeName(file.name)}" kind="${kind}">\n${file.text.replace(/<\/attachment>/g, '</ attachment>')}\n</attachment>`);
	}
	return parts.filter(Boolean).join('\n\n');
}

const SECTION = /<attachment name="([^"]*)"(?: kind="(file|voice)")?>\n([\s\S]*?)\n<\/attachment>/g;

/** The typed text and the attached files of a message written by composeMessage. */
export function splitAttachments(content: string): { text: string; files: Attachment[] } {
	const files: Attachment[] = [];
	const text = content
		.replace(SECTION, (_, name: string, kind: AttachmentKind | undefined, body: string) => {
			files.push({ name, text: body, kind: kind ?? 'file' });
			return '';
		})
		.trim();
	return { text, files };
}

export function formatBytes(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`;
	return `${(bytes / 1024).toFixed(bytes < 10 * 1024 ? 1 : 0)} KB`;
}
