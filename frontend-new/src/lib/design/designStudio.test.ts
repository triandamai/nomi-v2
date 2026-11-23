import { describe, expect, it } from 'vitest';
import {
	DEVICE_WIDTHS,
	PREVIEW_SOURCE,
	STUDIO_SOURCE,
	activeDirectionForPreview,
	adjustDirectionMessage,
	annotateMessage,
	annotationsOn,
	clampZoom,
	compareWith,
	elementPinDraft,
	elementsOnPage,
	feedbackDraft,
	feedbackTarget,
	fitZoom,
	highlightMessage,
	isDesignSystemPage,
	modeMessage,
	openFeedback,
	pickDirectionMessage,
	pickVersion,
	previewUrl,
	readPreviewMessage,
	refineMessage,
	splitCanvasPages,
	versionsNewestFirst,
	zoomLabel,
	zoomStep,
	type Selection,
	type StudioDirectionSet,
	type StudioFeedback,
	type StudioPreview,
	isPenPreview,
	mergePenDocuments,
	penUrl,
	toolForKey,
} from './designStudio';

const preview = (extra: Partial<StudioPreview> = {}): StudioPreview => ({
	version: 2,
	direction_id: 'editorial',
	title: 'Editorial, second pass',
	pages: [
		{ path: 'index.html', title: 'Home' },
		{ path: 'menu/index.html', title: 'Menu' },
	],
	elements: [
		{ id: 'site-header', kind: 'component', label: 'Site header', pages: ['index.html', 'menu/index.html'] },
		{ id: 'hero', kind: 'section', label: 'Hero', pages: ['index.html'] },
	],
	preview_token: 'ab'.repeat(32),
	published_url: null,
	created_at: '2026-11-15T10:00:00Z',
	...extra,
});

const said = (number: number, extra: Partial<StudioFeedback> = {}): StudioFeedback => ({
	id: `f${number}`,
	number,
	version: 2,
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

const hero: Selection = { id: 'hero', page: 'index.html', kind: 'section', label: 'Hero', tag: 'section', text: '', point: { x: 0.5, y: 0.25 } };

const directionSet: StudioDirectionSet = {
	version: 1,
	brief_version: 1,
	directions: [
		{
			id: 'editorial',
			name: 'Editorial',
			summary: 'Warm paper and serif display type.',
			rationale: 'Matches craft bakery positioning.',
			trade_offs: ['Needs strong photography'],
			palette: [
				{ name: 'Paper', hex: '#F6EFE4' },
				{ name: 'Accent', hex: '#B5532A' },
			],
			typography: { heading: 'Fraunces', body: 'Inter' },
			layout: 'Hero, daily bakes, pre-order',
			recommended: true,
		},
		{
			id: 'swiss',
			name: 'Swiss utility',
			summary: 'High-contrast grid with mono prices.',
			rationale: 'Fast scanning on mobile.',
			trade_offs: ['Less atmospheric'],
			palette: [
				{ name: 'Surface', hex: '#FFFFFF' },
				{ name: 'Ink', hex: '#111111' },
			],
			typography: { heading: 'Space Grotesk', body: 'IBM Plex Sans' },
			layout: 'Two-column menu table',
		},
	],
};

describe('designStudio', () => {
	it('addresses a version by its token, apart from the app', () => {
		expect(previewUrl(preview())).toBe(`/design-preview/${'ab'.repeat(32)}/index.html`);
		expect(previewUrl(preview(), 'menu/index.html')).toBe(`/design-preview/${'ab'.repeat(32)}/menu/index.html`);
		expect(previewUrl(preview(), '../../api/profile')).toBe(`/design-preview/${'ab'.repeat(32)}/index.html`);
	});

	it('opens the version asked for, else the newest', () => {
		const versions = [preview({ version: 1 }), preview({ version: 2 }), preview({ version: 3, direction_id: 'swiss' })];
		expect(pickVersion(versions, '2')?.version).toBe(2);
		expect(pickVersion(versions, null)?.version).toBe(3);
		expect(pickVersion(versions, '9')?.version).toBe(3);
		expect(pickVersion(versions, 'two')?.version).toBe(3);
		expect(pickVersion([], '1')).toBeNull();
		expect(versionsNewestFirst(versions).map((v) => v.version)).toEqual([3, 2, 1]);
	});

	it('lists the elements of one page and device widths', () => {
		expect(elementsOnPage(preview(), 'menu/index.html').map((e) => e.id)).toEqual(['site-header']);
		expect(elementsOnPage(preview(), 'index.html').map((e) => e.id)).toEqual(['site-header', 'hero']);
		expect(DEVICE_WIDTHS.desktop).toBeNull();
		expect(DEVICE_WIDTHS.tablet).toBeGreaterThan(DEVICE_WIDTHS.phone as number);
	});

	it('tells the preview when to pick elements and reads untrusted preview messages safely', () => {
		expect(modeMessage(true)).toEqual({ source: STUDIO_SOURCE, type: 'mode', select: true });
		expect(highlightMessage('hero')).toEqual({ source: STUDIO_SOURCE, type: 'highlight', id: 'hero' });
		const message = { source: PREVIEW_SOURCE, type: 'select', page: 'index.html', id: 'hero', tag: 'section', text: 'Fresh at six' };
		expect(readPreviewMessage(message, preview())).toEqual({
			type: 'select',
			selection: { id: 'hero', page: 'index.html', kind: 'section', label: 'Hero', tag: 'section', text: 'Fresh at six', point: null },
		});
		expect(readPreviewMessage({ ...message, source: 'someone-else' }, preview())).toBeNull();
		expect(readPreviewMessage({ ...message, id: 'missing' }, preview())).toBeNull();
	});

	it('drafts feedback, marks annotations, refines with Sora, and compares versions', () => {
		expect(feedbackDraft({ version: 2, scope: 'global', page: 'index.html', selection: hero, interaction: true, body: ' Warmer ' })).toEqual({
			version: 2,
			level: 'global',
			body: 'Warmer',
		});
		expect(openFeedback([said(1, { status: 'resolved' }), said(2), said(3)]).map((f) => f.number)).toEqual([3, 2]);
		expect(annotationsOn([said(1, { version: 1, pin: { x: 0.1, y: 0.2 } })], preview(), 'index.html')).toEqual([
			{ id: 'hero', number: 1, pin: { x: 0.1, y: 0.2 } },
		]);
		expect(annotateMessage([{ id: 'hero', number: 1, pin: null }])).toEqual({
			source: STUDIO_SOURCE,
			type: 'annotate',
			items: [{ id: 'hero', number: 1, pin: null }],
		});
		expect(feedbackTarget(said(1))).toEqual({ level: 'section', page: 'index.html', element: 'hero' });
		expect(refineMessage('', [said(2)], (numbers) => `Act on ${numbers}`)).toBe('@Sora Act on #2');
		const versions = [preview({ version: 1 }), preview({ version: 2 }), preview({ version: 4 })];
		expect(compareWith(versions, versions[2], 1)?.version).toBe(1);
		expect(compareWith(versions, versions[2], null)?.version).toBe(2);
	});

	it('supports 2D spatial canvas zoom, page splitting, direction matching, and direct pin drafting', () => {
		expect(clampZoom(0.1)).toBe(0.25);
		expect(clampZoom(3)).toBe(2);
		expect(zoomStep(1, 'in')).toBe(1.25);
		expect(zoomStep(1, 'out')).toBe(0.75);
		expect(zoomLabel(0.75)).toBe('75%');
		expect(fitZoom(1064, 2000)).toBe(0.5);

		const pages = [
			{ path: 'design-system.html', title: 'Design System & Tokens' },
			{ path: 'index.html', title: 'Home' },
			{ path: 'menu/index.html', title: 'Menu' },
		];
		expect(isDesignSystemPage(pages[0])).toBe(true);
		expect(splitCanvasPages(pages)).toEqual({
			systemPages: [{ path: 'design-system.html', title: 'Design System & Tokens' }],
			flowPages: [
				{ path: 'index.html', title: 'Home' },
				{ path: 'menu/index.html', title: 'Menu' },
			],
		});
		expect(activeDirectionForPreview(directionSet, preview({ direction_id: 'swiss' }))?.id).toBe('swiss');
		expect(activeDirectionForPreview(directionSet, null)?.id).toBe('editorial');
		expect(pickDirectionMessage(directionSet.directions[0])).toBe(
			'@Sora Use the "Editorial" direction (editorial) and build the design system and multi-page preview.',
		);
		expect(adjustDirectionMessage(directionSet.directions[0], ' Warmer accent ')).toBe(
			'@Sora On the "Editorial" direction (editorial): Warmer accent',
		);
		expect(elementPinDraft(2, hero, 'Tighter padding')).toEqual({
			version: 2,
			level: 'section',
			page: 'index.html',
			element_id: 'hero',
			body: 'Tighter padding',
			pin: { x: 0.5, y: 0.25 },
		});
	});
});

describe('the .pen canvas', () => {
	const pen = (id: string, variables?: unknown) => JSON.stringify({ version: '1.0', ...(variables ? { variables } : {}), children: [{ id, type: 'frame' }] });
	const base = { version: 1, direction_id: 'd', title: 't', elements: [], preview_token: 'ab', published_url: null, created_at: '' };

	it('serves a version\'s .pen file beside its pages', () => {
		expect(penUrl({ ...base, pages: [] }, 'canvas.pen')).toBe('/design-preview/ab/canvas.pen');
	});

	it('puts several .pen files side by side and skips what is not one', () => {
		const merged = JSON.parse(mergePenDocuments([pen('home', { a: 1 }), 'not json', pen('menu', { b: 2 }), '{"children":[]}']) as string);
		expect(merged.children.map((c: { id: string }) => c.id)).toEqual(['home', 'menu']);
		expect(merged.variables).toEqual({ a: 1, b: 2 });
		expect(mergePenDocuments(['', '{}'])).toBeNull();
	});

	it('keeps the themes of the .pen files, joining the values of an axis several files declare', () => {
		const themed = (id: string, themes: Record<string, string[]>) =>
			JSON.stringify({ version: '1.0', themes, children: [{ id, type: 'frame' }] });
		const merged = JSON.parse(
			mergePenDocuments([
				themed('home', { theme: ['light', 'dark'] }),
				pen('plain'),
				themed('menu', { theme: ['dark', 'contrast'], density: ['compact', 'comfy'] })
			]) as string
		);
		expect(merged.themes).toEqual({ theme: ['light', 'dark', 'contrast'], density: ['compact', 'comfy'] });
		expect(JSON.parse(mergePenDocuments([pen('home')]) as string).themes).toBeUndefined();
	});

	it('draws a version on the canvas unless it is made of HTML pages', () => {
		expect(isPenPreview({ ...base, pages: [{ path: 'design-system', title: 'System' }] })).toBe(true);
		expect(isPenPreview({ ...base, pages: [{ path: 'index.html', title: 'Home' }], pen_files: ['canvas.pen'] })).toBe(true);
		expect(isPenPreview({ ...base, pages: [{ path: 'index.html', title: 'Home' }] })).toBe(false);
		expect(isPenPreview(null)).toBe(false);
	});

	it('picks a tool by its key, but not while the person types or uses a shortcut', () => {
		expect(toolForKey({ key: 'v' })).toBe('select');
		expect(toolForKey({ key: 'H' })).toBe('move');
		expect(toolForKey({ key: 'c' })).toBe('comment');
		expect(toolForKey({ key: 'c', metaKey: true })).toBeNull();
		expect(toolForKey({ key: 'v', target: { tagName: 'TEXTAREA' } as never })).toBeNull();
		expect(toolForKey({ key: 'x' })).toBeNull();
	});
});
