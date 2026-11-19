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

	it('links the published address when the version has one', () => {
		const root = studio([preview(1, { published_url: 'https://ship.example.com/dpl/d1' })]);
		const link = root.querySelector('a[href="https://ship.example.com/dpl/d1"]') as HTMLAnchorElement;
		expect(link.getAttribute('rel')).toContain('noopener');
	});

	it('says when a version is not published outside Nomi, and why', () => {
		const root = studio([preview(1, { publish_error: 'publishing previews isn’t set up' })]);
		expect(text(root)).toContain('Not published outside Nomi: publishing previews isn’t set up');
		expect(button(root, 'Publish again')).toBeDefined();
	});

	it('says nothing about publishing when the version is published', () => {
		const root = studio([preview(1, { published_url: 'https://ship.example.com/dpl/d1', publish_error: null })]);
		expect(text(root)).not.toContain('Not published outside Nomi');
		expect(button(root, 'Publish again')).toBeUndefined();
	});

	it('publishes a version again and links it once it is out', async () => {
		const fetched = vi.fn(async (_url: string, _init?: RequestInit) => new Response(JSON.stringify({ version: 1, published_url: 'https://ship.example.com/dpl/d2', publish_error: null }), { status: 200 }));
		vi.stubGlobal('fetch', fetched);
		const root = studio([preview(1, { publish_error: 'Shipyard refused it' })]);
		button(root, 'Publish again').click();
		await settle();
		await settle();
		expect(fetched.mock.calls[0][0]).toBe('/chat/s1/design/publish');
		expect(JSON.parse(fetched.mock.calls[0][1]?.body as string)).toEqual({ version: 1 });
		expect(root.querySelector('a[href="https://ship.example.com/dpl/d2"]')).not.toBeNull();
		expect(text(root)).not.toContain('Not published outside Nomi');
	});

	it('keeps saying why when publishing again fails', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => new Response(JSON.stringify({ version: 1, published_url: null, publish_error: 'Shipyard is down' }), { status: 200 })));
		const root = studio([preview(1, { publish_error: 'Shipyard refused it' })]);
		button(root, 'Publish again').click();
		await settle();
		await settle();
		expect(text(root)).toContain('Not published outside Nomi: Shipyard is down');
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

	it('renders Stage 1 direction style-tile artboards before previews exist and lets the person pick or adjust a direction', async () => {
		const fetched = vi.fn(async (_url: string, _init?: RequestInit) => new Response(JSON.stringify({ ok: true }), { status: 202 }));
		vi.stubGlobal('fetch', fetched);
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
		expect(text(root)).toContain('Editorial');
		expect(text(root)).toContain('#F6EFE4');
		expect(text(root)).toContain('Fraunces');
		button(root, 'Use this direction').click();
		await settle();
		await settle();
		expect(fetched.mock.calls[0][0]).toBe('/chat/s1/design/refine');
		expect(JSON.parse(fetched.mock.calls[0][1]?.body as string)).toEqual({
			text: '@Sora Use the "Editorial" direction (editorial) and build the design system and multi-page preview.',
		});
	});

	it('renders the 2D spatial canvas with Design System and multi-page App Flow artboards and zoom controls by default', async () => {
		const root = studio([preview(1)]);
		expect(root.querySelector('[data-artboard="design-system"]')).not.toBeNull();
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

	it('opens a direct pin popover when an element is picked and sends batch revisions from the Revise with Sora bar', async () => {
		const fetched = vi.fn(async (url: string, _init?: RequestInit) => {
			if (url.endsWith('/feedback')) return new Response(JSON.stringify(said(1)), { status: 201 });
			return new Response(JSON.stringify({ ok: true }), { status: 202 });
		});
		vi.stubGlobal('fetch', fetched);
		const root = studio([preview(1)]);
		const homeFrame = root.querySelector('.studio__artboard--flow iframe') as HTMLIFrameElement;
		window.dispatchEvent(
			new MessageEvent('message', {
				source: homeFrame.contentWindow,
				data: { source: PREVIEW_SOURCE, type: 'select', page: 'index.html', id: 'hero', tag: 'section', text: 'Fresh bread', point: { x: 0.4, y: 0.3 } },
			}),
		);
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

	it('switches between OpenPencil vector canvas and iframe artboards', async () => {
		const root = studio([preview(1)]);
		expect(button(root, 'OpenPencil Vector')).toBeDefined();
		button(root, 'OpenPencil Vector').click();
		await settle();
		expect(root.querySelector('canvas')).not.toBeNull();
		button(root, 'Artboards').click();
		await settle();
		expect(root.querySelector('[data-artboard="design-system"]')).not.toBeNull();
	});
});


