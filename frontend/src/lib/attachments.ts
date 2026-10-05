// Files attached in the composer travel inside the message text, so every model provider can
// read them: each file becomes an <attachment name="…">…</attachment> section after the typed
// text. The chat shows those sections as file chips (see splitAttachments).

export interface Attachment {
	name: string;
	text: string;
}

export const MAX_ATTACHMENTS = 5;
export const MAX_FILE_BYTES = 100 * 1024;
export const MAX_TOTAL_BYTES = 200 * 1024;

const TEXT_EXTENSIONS = new Set([
	'txt', 'md', 'markdown', 'csv', 'tsv', 'json', 'yaml', 'yml', 'toml', 'xml', 'html', 'css', 'js', 'ts', 'jsx', 'tsx',
	'svelte', 'vue', 'py', 'rb', 'go', 'rs', 'java', 'kt', 'swift', 'c', 'h', 'cpp', 'cs', 'php', 'sh', 'sql', 'log', 'ini', 'env',
]);

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
		parts.push(`<attachment name="${escapeName(file.name)}">\n${file.text.replace(/<\/attachment>/g, '</ attachment>')}\n</attachment>`);
	}
	return parts.filter(Boolean).join('\n\n');
}

const SECTION = /<attachment name="([^"]*)">\n([\s\S]*?)\n<\/attachment>/g;

/** The typed text and the attached files of a message written by composeMessage. */
export function splitAttachments(content: string): { text: string; files: Attachment[] } {
	const files: Attachment[] = [];
	const text = content
		.replace(SECTION, (_, name: string, body: string) => {
			files.push({ name, text: body });
			return '';
		})
		.trim();
	return { text, files };
}

export function formatBytes(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`;
	return `${(bytes / 1024).toFixed(bytes < 10 * 1024 ? 1 : 0)} KB`;
}
