// The Design Studio: a chat's design previews (backend routes::design_studio), shown in a frame
// that has an origin of its own. The frame's page talks to the Studio only through messages,
// and nothing it says is taken on trust.

export type StudioElementKind = 'section' | 'component' | 'element';
export type StudioElement = { id: string; kind: StudioElementKind; label: string; pages: string[] };
export type StudioPage = { path: string; title: string };
export type StudioPreview = {
	version: number;
	direction_id: string;
	title: string;
	pages: StudioPage[];
	elements: StudioElement[];
	/** The unguessable name the version's files are served under. */
	preview_token: string;
	published_url: string | null;
	created_at: string;
};
export type StudioResponse = { previews: StudioPreview[] };

/** Who a message is from: the preview's page, or the Studio around it. */
export const PREVIEW_SOURCE = 'nomi-design-preview';
export const STUDIO_SOURCE = 'nomi-design-studio';

export type Device = 'desktop' | 'tablet' | 'phone';
/** How wide each device shows the preview, in CSS pixels; a desktop takes the room there is. */
export const DEVICE_WIDTHS: Record<Device, number | null> = { desktop: null, tablet: 768, phone: 390 };

const ENTRY_PAGE = 'index.html';
/** The most of an element's text the Studio keeps. */
const TEXT_CHARS = 120;
const TAG = /^[a-z][a-z0-9-]{0,19}$/;

/** Where a page of a version is served, for the frame. A page the version doesn't have opens its first. */
export function previewUrl(preview: StudioPreview, page: string = ENTRY_PAGE): string {
	const known = preview.pages.some((p) => p.path === page) ? page : ENTRY_PAGE;
	return `/design-preview/${preview.preview_token}/${known}`;
}

/** The version asked for (`?version=2`), else the newest; null when there are none. */
export function pickVersion(previews: StudioPreview[], wanted: string | null): StudioPreview | null {
	const asked = previews.find((p) => String(p.version) === wanted);
	return asked ?? previews.reduce<StudioPreview | null>((newest, p) => (!newest || p.version > newest.version ? p : newest), null);
}

export function versionsNewestFirst(previews: StudioPreview[]): StudioPreview[] {
	return [...previews].sort((a, b) => b.version - a.version);
}

export function elementsOnPage(preview: StudioPreview, page: string): StudioElement[] {
	return preview.elements.filter((element) => element.pages.includes(page));
}

/** Tells the preview whether a click picks an element or does what the page does. */
export function modeMessage(select: boolean) {
	return { source: STUDIO_SOURCE, type: 'mode' as const, select };
}

/** Tells the preview which element to outline; null clears it. */
export function highlightMessage(id: string | null) {
	return { source: STUDIO_SOURCE, type: 'highlight' as const, id };
}

/** An element the person picked in the preview. */
export type Selection = { id: string; page: string; kind: StudioElementKind; label: string; tag: string; text: string };
export type PreviewMessage = { type: 'ready'; page: string } | { type: 'select'; selection: Selection };

/**
 * What the preview's page said, if it is something the Studio understands about this version:
 * a page the version has, and an element its manifest lists. Anything else is null. The
 * element's kind and label come from the manifest, never from the message.
 */
export function readPreviewMessage(data: unknown, preview: StudioPreview): PreviewMessage | null {
	if (typeof data !== 'object' || data === null) return null;
	const said = data as Record<string, unknown>;
	if (said.source !== PREVIEW_SOURCE) return null;
	const page = said.page;
	if (typeof page !== 'string' || !preview.pages.some((p) => p.path === page)) return null;
	if (said.type === 'ready') return { type: 'ready', page };
	if (said.type !== 'select') return null;
	const element = preview.elements.find((e) => e.id === said.id);
	if (!element) return null;
	const tag = typeof said.tag === 'string' && TAG.test(said.tag) ? said.tag : '';
	const text = typeof said.text === 'string' ? said.text.slice(0, TEXT_CHARS) : '';
	return { type: 'select', selection: { id: element.id, page, kind: element.kind, label: element.label, tag, text } };
}
