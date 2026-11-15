import { describe, expect, it } from 'vitest';
import {
	DEVICE_WIDTHS,
	PREVIEW_SOURCE,
	STUDIO_SOURCE,
	elementsOnPage,
	highlightMessage,
	modeMessage,
	pickVersion,
	previewUrl,
	readPreviewMessage,
	versionsNewestFirst,
	type StudioPreview,
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
		expect(versions.map((v) => v.version)).toEqual([1, 2, 3]);
	});

	it('lists the elements of one page', () => {
		expect(elementsOnPage(preview(), 'menu/index.html').map((e) => e.id)).toEqual(['site-header']);
		expect(elementsOnPage(preview(), 'index.html').map((e) => e.id)).toEqual(['site-header', 'hero']);
	});

	it('shows a phone and a tablet at their widths and a desktop at full width', () => {
		expect(DEVICE_WIDTHS.desktop).toBeNull();
		expect(DEVICE_WIDTHS.tablet).toBeGreaterThan(DEVICE_WIDTHS.phone as number);
	});

	it('tells the preview when to pick elements and which one to outline', () => {
		expect(modeMessage(true)).toEqual({ source: STUDIO_SOURCE, type: 'mode', select: true });
		expect(highlightMessage('hero')).toEqual({ source: STUDIO_SOURCE, type: 'highlight', id: 'hero' });
		expect(highlightMessage(null)).toEqual({ source: STUDIO_SOURCE, type: 'highlight', id: null });
	});

	it('reads what the preview says about an element the version has', () => {
		const said = { source: PREVIEW_SOURCE, type: 'select', page: 'index.html', id: 'hero', tag: 'section', text: 'Fresh at six', rect: { x: 0, y: 10, width: 300, height: 200 } };
		expect(readPreviewMessage(said, preview())).toEqual({
			type: 'select',
			selection: { id: 'hero', page: 'index.html', kind: 'section', label: 'Hero', tag: 'section', text: 'Fresh at six', point: null },
		});
		expect(readPreviewMessage({ source: PREVIEW_SOURCE, type: 'ready', page: 'menu/index.html' }, preview())).toEqual({ type: 'ready', page: 'menu/index.html' });
	});

	it('takes nothing on trust from the preview', () => {
		const select = { source: PREVIEW_SOURCE, type: 'select', page: 'index.html', id: 'hero', tag: 'section', text: 'Hi' };
		for (const bad of [
			null,
			'select',
			{ ...select, source: 'someone-else' },
			{ ...select, id: 'not-in-this-version' },
			{ ...select, id: 42 },
			{ ...select, page: 'admin.html' },
			{ ...select, type: 'approve' },
			{ source: PREVIEW_SOURCE, type: 'ready', page: 'https://evil.example/' },
		]) {
			expect(readPreviewMessage(bad, preview())).toBeNull();
		}
		const messy = readPreviewMessage({ ...select, tag: '<script>', text: 'x'.repeat(500) }, preview());
		expect(messy).toMatchObject({ type: 'select', selection: { tag: '', text: 'x'.repeat(120) } });
		expect(readPreviewMessage({ ...select, text: { toString: () => 'boo' } }, preview())).toMatchObject({ selection: { text: '' } });
	});
});

import { annotateMessage, annotationsOn, feedbackDraft, feedbackTarget, openFeedback, type Selection, type StudioFeedback } from './designStudio';

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

describe('designStudio feedback', () => {
	it('drafts feedback about the whole design, a page or the selected element', () => {
		expect(feedbackDraft({ version: 2, scope: 'global', page: 'index.html', selection: hero, interaction: true, body: ' Warmer ' })).toEqual({ version: 2, level: 'global', body: 'Warmer' });
		expect(feedbackDraft({ version: 2, scope: 'page', page: 'menu/index.html', selection: null, interaction: false, body: 'Too long' })).toEqual({ version: 2, level: 'page', page: 'menu/index.html', body: 'Too long' });
		expect(feedbackDraft({ version: 2, scope: 'element', page: 'index.html', selection: hero, interaction: false, body: 'Shorter' })).toEqual({
			version: 2,
			level: 'section',
			page: 'index.html',
			element_id: 'hero',
			body: 'Shorter',
			pin: { x: 0.5, y: 0.25 },
		});
	});

	it('drafts feedback about how a page or an element behaves', () => {
		expect(feedbackDraft({ version: 2, scope: 'page', page: 'index.html', selection: null, interaction: true, body: 'Slide in' })).toEqual({ version: 2, level: 'interaction', page: 'index.html', body: 'Slide in' });
		const onElement = feedbackDraft({ version: 2, scope: 'element', page: 'index.html', selection: { ...hero, point: null }, interaction: true, body: 'Fade' });
		expect(onElement).toEqual({ version: 2, level: 'interaction', page: 'index.html', element_id: 'hero', body: 'Fade' });
	});

	it('drafts nothing without words, or about an element that is not picked', () => {
		expect(feedbackDraft({ version: 2, scope: 'global', page: 'index.html', selection: null, interaction: false, body: '  ' })).toBeNull();
		expect(feedbackDraft({ version: 2, scope: 'element', page: 'index.html', selection: null, interaction: false, body: 'x' })).toBeNull();
		expect(feedbackDraft({ version: 2, scope: 'element', page: 'menu/index.html', selection: hero, interaction: false, body: 'x' })).toBeNull();
	});

	it('keeps the open feedback, newest first', () => {
		const all = [said(1, { status: 'resolved' }), said(2), said(3)];
		expect(openFeedback(all).map((f) => f.number)).toEqual([3, 2]);
	});

	it('marks the open feedback that points at an element this page of this version still has', () => {
		const all = [
			said(1, { version: 1, pin: { x: 0.1, y: 0.2 } }),
			said(2, { status: 'resolved' }),
			said(3, { element_id: 'gone' }),
			said(4, { level: 'global', page: null, element_id: null }),
			said(5, { level: 'interaction', element_id: 'site-header', page: 'menu/index.html' }),
		];
		expect(annotationsOn(all, preview(), 'index.html')).toEqual([
			{ id: 'hero', number: 1, pin: { x: 0.1, y: 0.2 } },
			{ id: 'site-header', number: 5, pin: null },
		]);
		expect(annotateMessage([{ id: 'hero', number: 1, pin: null }])).toEqual({ source: STUDIO_SOURCE, type: 'annotate', items: [{ id: 'hero', number: 1, pin: null }] });
	});

	it('says what feedback points at', () => {
		expect(feedbackTarget(said(1, { level: 'global', page: null, element_id: null }))).toEqual({ level: 'global', page: null, element: null });
		expect(feedbackTarget(said(1))).toEqual({ level: 'section', page: 'index.html', element: 'hero' });
	});

	it('takes the point clicked from the preview only when it is inside the element', () => {
		const from = (point: unknown) => readPreviewMessage({ source: PREVIEW_SOURCE, type: 'select', page: 'index.html', id: 'hero', tag: 'section', text: '', point }, preview());
		expect(from({ x: 0.5, y: 0.25 })).toMatchObject({ selection: { point: { x: 0.5, y: 0.25 } } });
		for (const off of [{ x: 2, y: 0 }, { x: 'a', y: 0 }, null, undefined, { x: -1, y: 0.5 }]) expect(from(off)).toMatchObject({ selection: { point: null } });
	});
});

import { refineMessage } from './designStudio';

describe('designStudio refinement', () => {
	const fallback = (numbers: string) => `Please act on feedback ${numbers}.`;

	it('asks Sora by name, in the person’s words when they gave some', () => {
		expect(refineMessage('  Only the hero for now ', [said(2)], fallback)).toBe('@Sora Only the hero for now');
	});

	it('asks for the open feedback by number when the person said nothing', () => {
		expect(refineMessage('', [said(1, { status: 'resolved' }), said(2), said(5)], fallback)).toBe('@Sora Please act on feedback #2, #5.');
	});

	it('asks nothing when there is nothing to ask', () => {
		expect(refineMessage(' ', [said(1, { status: 'resolved' })], fallback)).toBeNull();
	});
});

import { compareWith } from './designStudio';

describe('designStudio compare', () => {
	const versions = [preview({ version: 1 }), preview({ version: 2 }), preview({ version: 4 })];

	it('compares a version with the one picked', () => {
		expect(compareWith(versions, versions[2], 1)?.version).toBe(1);
	});

	it('compares with the version before when none is picked, or the pick is the version itself', () => {
		expect(compareWith(versions, versions[2], null)?.version).toBe(2);
		expect(compareWith(versions, versions[1], 2)?.version).toBe(1);
		expect(compareWith(versions, versions[1], 9)?.version).toBe(1);
	});

	it('compares the first version with the next one, and a lone version with nothing', () => {
		expect(compareWith(versions, versions[0], null)?.version).toBe(2);
		expect(compareWith([versions[0]], versions[0], null)).toBeNull();
	});
});
