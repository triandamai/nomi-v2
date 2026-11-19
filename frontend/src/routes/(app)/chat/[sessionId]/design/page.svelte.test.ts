// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { render, settle, text } from '../../../../../test-stubs/render';
import Page from './+page.svelte';
import { PREVIEW_SOURCE, type StudioApproval, type StudioFeedback, type StudioPreview } from '$lib/designStudio';

const TOKEN = 'ab'.repeat(32);
const preview = (version: number, extra: Partial<StudioPreview> = {}): StudioPreview => ({
	version,
	direction_id: 'editorial',
	title: `Editorial pass ${version}`,
	pages: [
		{ path: 'index.html', title: 'Home' },
		{ path: 'menu/index.html', title: 'Menu' },
	],
	elements: [
		{ id: 'hero', kind: 'section', label: 'Hero', pages: ['index.html'] },
		{ id: 'menu-list', kind: 'component', label: 'Menu list', pages: ['menu/index.html'] },
	],
	preview_token: TOKEN,
	published_url: null,
	created_at: '2026-11-15T10:00:00Z',
	...extra,
});

const studio = (previews: StudioPreview[], wanted: string | null = null, feedback: StudioFeedback[] = [], approval: StudioApproval | null = null) =>
	render(Page as never, { data: { previews, feedback, approval, wanted, sessionId: 's1' } as never, params: { sessionId: 's1' } as never });

const said = (number: number, extra: Partial<StudioFeedback> = {}): StudioFeedback => ({
	id: `f${number}`,
	number,
	version: 1,
	level: 'section',
	page: 'index.html',
	element_id: 'hero',
	body: 'Make it shorter',
	pin: null,
	status: 'open',
	resolved_in_version: null,
	resolution: null,
	created_at: '2026-11-15T10:00:00Z',
	resolved_at: null,
	...extra,
});

const button = (root: HTMLElement, label: string) => [...root.querySelectorAll('button')].find((b) => b.textContent?.trim() === label) as HTMLButtonElement;

async function type(root: HTMLElement, words: string) {
	const box = root.querySelector('textarea') as HTMLTextAreaElement;
	box.value = words;
	box.dispatchEvent(new Event('input', { bubbles: true }));
	await settle();
}

afterEach(() => vi.unstubAllGlobals());

const frame = (root: HTMLElement) => root.querySelector('iframe') as HTMLIFrameElement;

describe('design studio page', () => {
	it('says so when there is nothing to look at yet', () => {
		const root = studio([]);
		expect(text(root)).toContain('No previews yet');
		expect(frame(root)).toBeNull();
	});

	it('shows the newest version in artboard frames that have an origin of their own', () => {
		const root = studio([preview(1), preview(2)]);
		const shown = frame(root);
		expect(shown.getAttribute('src')).toBe(`/design-preview/${TOKEN}/index.html`);
		expect(shown.getAttribute('sandbox')).toBe('allow-scripts');
		expect(shown.getAttribute('referrerpolicy')).toBe('no-referrer');
		expect(text(root)).toContain('Editorial pass 2');
	});

	it('opens the version the link asked for', async () => {
		const root = studio([preview(1), preview(2, { preview_token: 'cd'.repeat(32) })], '1');
		expect(frame(root).getAttribute('src')).toBe(`/design-preview/${TOKEN}/index.html`);
		expect(text(root)).toContain('Editorial pass 1');
	});

	it('sizes the artboard for a phone and back', async () => {
		const root = studio([preview(1)]);
		const phone = root.querySelector('button[aria-label^="Phone"]') as HTMLButtonElement;
		phone.click();
		await settle();
		const board = root.querySelector('.studio__artboard--flow') as HTMLElement;
		expect(board.style.width).toBe('390px');
		(root.querySelector('button[aria-label="Desktop"]') as HTMLButtonElement).click();
		await settle();
		expect(board.style.width).toBe('1120px');
	});

	it('ignores a message that is not from its own frame', async () => {
		const root = studio([preview(1)]);
		window.dispatchEvent(new MessageEvent('message', { data: { source: PREVIEW_SOURCE, type: 'select', page: 'index.html', id: 'hero', tag: 'h1', text: 'Hi' }, source: window }));
		await settle();
		expect(root.querySelector('.studio__pin-popover')).toBeNull();
	});

	it('never offers to publish a version', () => {
		const root = studio([preview(1, { publish_error: 'Shipyard refused it' })]);
		expect(text(root)).not.toContain('Not published outside Nomi');
		expect(button(root, 'Publish again')).toBeUndefined();
	});

	it('approves a version only after the person confirms', async () => {
		const fetched = vi.fn(async (_url: string, _init?: RequestInit) => new Response(JSON.stringify({ version: 2, note: null, created_at: '2026-11-15T10:00:00Z' }), { status: 201 }));
		vi.stubGlobal('fetch', fetched);
		const root = studio([preview(1), preview(2)], null, [said(1)]);
		button(root, 'Approve this version').click();
		await settle();
		expect(fetched).not.toHaveBeenCalled();
		expect(text(root)).toContain('Approve version 2 as the design to build?');
		expect(text(root)).toContain('Feedback still waiting: 1');
		button(root, 'Approve version 2').click();
		await settle();
		await settle();
		expect(fetched.mock.calls[0][0]).toBe('/chat/s1/design/approval');
		expect(JSON.parse(fetched.mock.calls[0][1]?.body as string)).toEqual({ version: 2 });
		expect(button(root, 'Approve this version')).toBeUndefined();
		expect(root.querySelector('.studio__approved')?.textContent).toContain('Approved');
	});

	it('shows when the current version is approved', async () => {
		const root = studio([preview(1)], null, [], { version: 1, note: null, created_at: '2026-11-15T10:00:00Z' });
		expect(button(root, 'Approve this version')).toBeUndefined();
		expect(root.querySelector('.studio__approved')?.textContent).toContain('Approved');
	});

	it('cancels an approval that was not confirmed', async () => {
		const root = studio([preview(1)]);
		button(root, 'Approve this version').click();
		await settle();
		button(root, 'Cancel').click();
		await settle();
		expect(text(root)).not.toContain('as the design to build?');
		expect(root.querySelector('.studio__approved')).toBeNull();
	});

	it('draws the proposed directions on the vector canvas before previews exist, with the canvas tools', () => {
		const directions = {
			version: 1,
			brief_version: 1,
			directions: [
				{
					id: 'editorial',
					name: 'Editorial',
					summary: 'Warm paper tones and serif headings.',
					rationale: 'Fits craft bakery story.',
					trade_offs: ['Needs strong photos'],
					palette: [
						{ name: 'Paper', hex: '#F6EFE4' },
						{ name: 'Accent', hex: '#B5532A' },
					],
					typography: { heading: 'Fraunces', body: 'Inter' },
					layout: 'Hero, daily bakes, pre-order',
					recommended: true,
				},
			],
		};
		const root = render(Page as never, {
			data: { directions, previews: [], feedback: [], approval: null, wanted: null, sessionId: 's1' } as never,
			params: { sessionId: 's1' } as never,
		});
		expect(root.querySelector('canvas[data-testid="openpencil-canvas"]')).not.toBeNull();
		expect(root.querySelector('[role="toolbar"]')).not.toBeNull();
		expect(button(root, 'All boards')).toBeUndefined();
		expect(button(root, 'Artboards')).toBeUndefined();
	});

	it("shows an HTML version's pages as artboards with zoom controls", async () => {
		const root = studio([preview(1)]);
		const flowFrames = [...root.querySelectorAll('.studio__artboard--flow iframe')];
		expect(flowFrames.map((f) => f.getAttribute('src'))).toEqual([
			`/design-preview/${TOKEN}/index.html`,
			`/design-preview/${TOKEN}/menu/index.html`,
		]);
		expect(text(root)).toContain('100%');
		(root.querySelector('button[aria-label="Zoom out"]') as HTMLButtonElement).click();
		await settle();
		expect(text(root)).toContain('75%');
	});

	it('draws a .pen version from its own file and never frames its pages as HTML', async () => {
		const pen = JSON.stringify({ version: '1.0', children: [{ id: 'design-system', type: 'frame', name: 'Design system', width: 390, height: 600, children: [{ id: 'buttons', type: 'frame', name: 'Buttons', width: 390, height: 120 }] }] });
		const fetched = vi.fn(async (_url: string) => new Response(pen, { status: 200 }));
		vi.stubGlobal('fetch', fetched);
		const root = studio([
			preview(1, {
				pages: [{ path: 'design-system', title: 'Design system' }],
				elements: [{ id: 'buttons', kind: 'component', label: 'Buttons', pages: ['design-system'] }],
				pen_files: ['canvas.pen'],
			}),
		]);
		await settle();
		await settle();
		expect(fetched.mock.calls[0][0]).toBe(`/design-preview/${TOKEN}/canvas.pen`);
		expect(root.querySelector('iframe')).toBeNull();
		expect(root.querySelector('canvas[data-testid="openpencil-canvas"]')).not.toBeNull();
	});

	it('picks the canvas tool with a click or its key, as in a design tool', async () => {
		const root = studio([preview(1)]);
		const pressed = () => root.querySelector('[role="toolbar"] button[aria-pressed="true"]')?.getAttribute('aria-label');
		expect(pressed()).toBe('Select (V)');
		window.dispatchEvent(new KeyboardEvent('keydown', { key: 'h' }));
		await settle();
		expect(pressed()).toBe('Move (H)');
		(root.querySelector('button[aria-label="Comment (C)"]') as HTMLButtonElement).click();
		await settle();
		expect(pressed()).toBe('Comment (C)');
	});

	it('selects an element without opening a note, and pins a note with the comment tool', async () => {
		const fetched = vi.fn(async (url: string, _init?: RequestInit) => {
			if (url.endsWith('/feedback')) return new Response(JSON.stringify(said(1)), { status: 201 });
			return new Response(JSON.stringify({ ok: true }), { status: 202 });
		});
		vi.stubGlobal('fetch', fetched);
		const root = studio([preview(1)]);
		const homeFrame = root.querySelector('.studio__artboard--flow iframe') as HTMLIFrameElement;
		const pick = () =>
			window.dispatchEvent(
				new MessageEvent('message', {
					source: homeFrame.contentWindow,
					data: { source: PREVIEW_SOURCE, type: 'select', page: 'index.html', id: 'hero', tag: 'section', text: 'Fresh bread', point: { x: 0.4, y: 0.3 } },
				}),
			);
		pick();
		await settle();
		expect(root.querySelector('.studio__pin-popover')).toBeNull();
		window.dispatchEvent(new KeyboardEvent('keydown', { key: 'c' }));
		await settle();
		pick();
		await settle();
		expect(root.querySelector('.studio__pin-popover')).not.toBeNull();
		await type(root, 'Make it shorter');
		button(root, 'Save note').click();
		await settle();
		await settle();
		expect(fetched.mock.calls[0][0]).toBe('/chat/s1/design/feedback');
		expect(root.querySelector('.studio__revise-bar')).not.toBeNull();
		button(root, 'Revise with Sora').click();
		await settle();
		await settle();
		expect(fetched.mock.calls[1][0]).toBe('/chat/s1/design/refine');
	});
});
