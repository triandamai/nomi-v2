// Files in a chat, as the backend keeps them (backend/crates/nomi-server/src/routes/attachments.rs):
// each file is uploaded on its own as soon as it's picked (`POST /attachments`), and the message
// carries one reference tag per file after the typed text,
// <attachment id="…" name="…" kind="…" mime="…" size="…"/>. The backend checks the tags against
// the sender's own uploads and reads the files. These are the pure parts: the tags, the formats the
// chips use, why an upload failed, what keeps Send off, and which models can see an image.

export type AttachmentKind = 'text' | 'pdf' | 'document' | 'spreadsheet' | 'presentation' | 'image' | 'audio' | 'voice' | 'video' | 'other';

/** A file in a sent message, read from its tag. */
export type SentFile = { id: string; name: string; kind: AttachmentKind; mime?: string; size?: number };

/** `POST /api/attachments`: what the backend says about an upload. */
export type UploadedFile = { id: string; name: string; kind: AttachmentKind; mime: string; size: number; status: 'pending' | 'ready' | 'failed'; reference: string };

/** Why an upload didn't make it. `connection_lost` is the browser's own; the rest are the status. */
export type UploadFailure = 'connection_lost' | 'storage_full' | 'too_large' | 'server_error' | 'rejected';

/** A file in the composer. */
export type ChipState = {
	key: string;
	name: string;
	size: number;
	status: 'uploading' | 'ready' | 'failed' | 'rejected' | 'warning';
	/** 0 to 1 while uploading. */
	progress?: number;
	failure?: UploadFailure;
	/** Uploading again after a failure: the restart copy (no resumable uploads, M1). */
	again?: boolean;
	uploaded?: UploadedFile;
};

/** The typed text followed by each file's reference tag. */
export function composeMessage(text: string, references: string[]): string {
	return [text.trim(), ...references].filter(Boolean).join('\n\n');
}

const REFERENCE = /<attachment ([^<>]*?)\/>/g;

function attribute(attributes: string, name: string): string | undefined {
	return new RegExp(`(?:^|\\s)${name}="([^"]*)"`).exec(attributes)?.[1];
}

/** The typed text and the files of a message. */
export function splitAttachments(content: string): { text: string; files: SentFile[] } {
	const files: SentFile[] = [];
	const text = content.replace(REFERENCE, (whole, attributes: string) => {
		const id = attribute(attributes, 'id');
		if (!id) return whole;
		const size = Number(attribute(attributes, 'size'));
		files.push({
			id,
			name: attribute(attributes, 'name') ?? 'file',
			kind: (attribute(attributes, 'kind') as AttachmentKind | undefined) ?? 'other',
			mime: attribute(attributes, 'mime'),
			size: Number.isFinite(size) ? size : undefined,
		});
		return '';
	});
	return { text: text.trim(), files };
}

const KB = 1024;
const MB = 1024 * KB;

const decimal = (value: number, places: number) => {
	const fixed = value.toFixed(places).replace(/\.0+$/, '');
	return fixed.replace('.', ',');
};

/** "4,2 MB", "18 KB": sizes with a decimal comma, as the pen writes them. */
export function formatBytes(bytes: number): string {
	if (bytes < KB) return `${bytes} B`;
	if (bytes < MB) return `${decimal(bytes / KB, bytes < 10 * KB ? 1 : 0)} KB`;
	return `${decimal(bytes / MB, bytes < 10 * MB ? 1 : 0)} MB`;
}

/** The parts of "42% · 2,1 of 5 MB": the bytes really sent, in the total's unit. */
export function progressParts(fraction: number, total: number): { percent: string; sent: string; total: string } {
	const share = Math.min(1, Math.max(0, fraction));
	const [unit, size] = total >= MB ? ['MB', MB] : total >= KB ? ['KB', KB] : ['B', 1];
	const sent = share * total;
	return { percent: String(Math.round(share * 100)), sent: sent === 0 ? '0' : decimal(sent / size, 1), total: `${decimal(total / size, total < 10 * size ? 1 : 0)} ${unit}` };
}

/** "42% · 2,1 of 5 MB", in English (the app writes it through its message). */
export function progressDetail(fraction: number, total: number): string {
	const parts = progressParts(fraction, total);
	return `${parts.percent}% · ${parts.sent} of ${parts.total}`;
}

/** ".dmg", or "" for a name without one. */
export function extensionOf(name: string): string {
	const dot = name.lastIndexOf('.');
	return dot > 0 ? name.slice(dot).toLowerCase() : '';
}

/** "PDF", "JPG": the extension in capitals, else the kind. */
export function kindLabel(file: { name: string; kind: string }): string {
	return (extensionOf(file.name).slice(1) || file.kind).toUpperCase();
}

export function uploadFailure(status: number): UploadFailure {
	if (status === 0) return 'connection_lost';
	if (status === 507) return 'storage_full';
	if (status === 413) return 'too_large';
	if (status === 415) return 'rejected';
	return 'server_error';
}

export type Blocked = { kind: 'rejected'; name: string } | { kind: 'failed'; count: number } | { kind: 'warning' } | { kind: 'uploading' };

/** Why Send is off, the one to fix first; null when every file is ready. */
export function blockedReason(chips: ChipState[]): Blocked | null {
	const rejected = chips.find((chip) => chip.status === 'rejected');
	if (rejected) return { kind: 'rejected', name: rejected.name };
	const failed = chips.filter((chip) => chip.status === 'failed').length;
	if (failed > 0) return { kind: 'failed', count: failed };
	if (chips.some((chip) => chip.status === 'warning')) return { kind: 'warning' };
	if (chips.some((chip) => chip.status === 'uploading')) return { kind: 'uploading' };
	return null;
}

/** Files that will go with the message (an image the model can't see still counts as made it). */
export function readyCount(chips: ChipState[]): number {
	return chips.filter((chip) => chip.status === 'ready' || chip.status === 'warning').length;
}

type ModelLike = { key: string; label: string; enabled: boolean; capabilities?: string[] };
type GroupLike<M extends ModelLike> = { source: 'nomi' | 'connection'; label: string; models: M[] };

/** Whether a model can see images. Unknown capabilities aren't held against it. */
export function seesImages(model: { capabilities?: string[] } | undefined): boolean {
	return !model?.capabilities || model.capabilities.includes('vision');
}

/** The switched-on models that can see images, by who pays, and the labels of those that can't. */
export function visionSplit<M extends ModelLike, G extends GroupLike<M>>(groups: G[], current: string | null): { groups: G[]; hidden: string[] } {
	const hidden: string[] = [];
	const kept = groups
		.map((group) => {
			const models = group.models.filter((model) => model.enabled || model.key === current);
			for (const model of models) if (!seesImages(model)) hidden.push(model.label);
			return { ...group, models: models.filter((model) => seesImages(model)) };
		})
		.filter((group) => group.models.length > 0);
	kept.sort((a, b) => (a.source === b.source ? 0 : a.source === 'connection' ? -1 : 1));
	return { groups: kept, hidden };
}

/** "deepseek-v3 and 2 others": the model in use first. */
export function hiddenLine(hidden: string[], current: string): { name: string; others: number } {
	const name = hidden.includes(current) ? current : (hidden[0] ?? current);
	return { name, others: Math.max(0, hidden.length - 1) };
}
