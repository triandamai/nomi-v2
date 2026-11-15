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
export type FeedbackLevel = 'global' | 'page' | 'section' | 'component' | 'element' | 'interaction';
/** A point inside what feedback is about: x across and y down, each from 0 to 1. */
export type StudioPin = { x: number; y: number };
export type StudioFeedback = {
	id: string;
	/** Its place in the chat's feedback: what Sora and the person call it. */
	number: number;
	/** The version it was given on. */
	version: number;
	level: FeedbackLevel;
	page: string | null;
	element_id: string | null;
	body: string;
	pin: StudioPin | null;
	status: 'open' | 'resolved';
	resolved_in_version: number | null;
	resolution: string | null;
	created_at: string;
	resolved_at: string | null;
};
/** The version the person said is the design to build. */
export type StudioApproval = { version: number; note: string | null; created_at: string };
export type StudioResponse = { previews: StudioPreview[]; feedback: StudioFeedback[]; approval: StudioApproval | null };

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
export type Selection = { id: string; page: string; kind: StudioElementKind; label: string; tag: string; text: string; point: StudioPin | null };
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
	return { type: 'select', selection: { id: element.id, page, kind: element.kind, label: element.label, tag, text, point: pointOf(said.point) } };
}

const inside = (value: unknown): value is number => typeof value === 'number' && value >= 0 && value <= 1;

function pointOf(point: unknown): StudioPin | null {
	if (typeof point !== 'object' || point === null) return null;
	const { x, y } = point as Record<string, unknown>;
	return inside(x) && inside(y) ? { x, y } : null;
}

/** What feedback is about, as the person picks it: everything, the page on screen, or the selected element. */
export type FeedbackScope = 'global' | 'page' | 'element';
export type FeedbackDraft = { version: number; level: FeedbackLevel; page?: string; element_id?: string; body: string; pin?: StudioPin };

/**
 * The feedback to send, or null when there is nothing to say or nothing picked to say it about.
 * `interaction` is for how the page or the element behaves rather than how it looks.
 */
export function feedbackDraft(input: { version: number; scope: FeedbackScope; page: string; selection: Selection | null; interaction: boolean; body: string }): FeedbackDraft | null {
	const body = input.body.trim();
	if (!body) return null;
	const { version, page, selection } = input;
	if (input.scope === 'global') return { version, level: 'global', body };
	if (input.scope === 'page') return { version, level: input.interaction ? 'interaction' : 'page', page, body };
	if (!selection || selection.page !== page) return null;
	const draft: FeedbackDraft = { version, level: input.interaction ? 'interaction' : selection.kind, page, element_id: selection.id, body };
	if (selection.point && !input.interaction) draft.pin = selection.point;
	return draft;
}

/** The feedback no version has dealt with yet, newest first. */
export function openFeedback(feedback: StudioFeedback[]): StudioFeedback[] {
	return feedback.filter((f) => f.status === 'open').sort((a, b) => b.number - a.number);
}

export function feedbackTarget(feedback: StudioFeedback): { level: FeedbackLevel; page: string | null; element: string | null } {
	return { level: feedback.level, page: feedback.page, element: feedback.element_id };
}

/** A numbered mark on an element of the preview. */
export type Annotation = { id: string; number: number; pin: StudioPin | null };

/**
 * The marks for a page of a version: open feedback about an element the page still has, whichever
 * version it was given on (an element keeps its id from one version to the next).
 */
export function annotationsOn(feedback: StudioFeedback[], preview: StudioPreview, page: string): Annotation[] {
	const here = new Set(elementsOnPage(preview, page).map((element) => element.id));
	return feedback
		.filter((f) => f.status === 'open' && f.element_id !== null && here.has(f.element_id))
		.map((f) => ({ id: f.element_id as string, number: f.number, pin: f.pin }));
}

/** Tells the preview which elements to mark with a feedback number. */
export function annotateMessage(items: Annotation[]) {
	return { source: STUDIO_SOURCE, type: 'annotate' as const, items };
}

/** The designer in the crew, as a message addresses her. */
const DESIGNER = 'Sora';

/**
 * The chat message that asks Sora for a new version: the person's words when they gave some,
 * else a request for the feedback still waiting, by number. Null when there is nothing to ask.
 */
export function refineMessage(words: string, feedback: StudioFeedback[], fallback: (numbers: string) => string): string | null {
	const said = words.trim();
	if (said) return `@${DESIGNER} ${said}`;
	const waiting = feedback.filter((f) => f.status === 'open').sort((a, b) => a.number - b.number);
	if (waiting.length === 0) return null;
	return `@${DESIGNER} ${fallback(waiting.map((f) => `#${f.number}`).join(', '))}`;
}

/**
 * The version to show next to `current`: the one picked, else the version before it, else the
 * one after. Null when there is no other version.
 */
export function compareWith(previews: StudioPreview[], current: StudioPreview, picked: number | null): StudioPreview | null {
	const others = previews.filter((p) => p.version !== current.version).sort((a, b) => a.version - b.version);
	const asked = others.find((p) => p.version === picked);
	if (asked) return asked;
	const before = others.filter((p) => p.version < current.version);
	return before[before.length - 1] ?? others[0] ?? null;
}
