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
			selection: { id: 'hero', page: 'index.html', kind: 'section', label: 'Hero', tag: 'section', text: 'Fresh at six' },
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
