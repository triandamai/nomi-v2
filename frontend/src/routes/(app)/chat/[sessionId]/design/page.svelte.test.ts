// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import { render, settle, text } from '../../../../../test-stubs/render';
import Page from './+page.svelte';
import { PREVIEW_SOURCE, type StudioPreview } from '$lib/designStudio';

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

const studio = (previews: StudioPreview[], wanted: string | null = null) =>
	render(Page as never, { data: { previews, wanted, sessionId: 's1' } as never, params: { sessionId: 's1' } as never });

const frame = (root: HTMLElement) => root.querySelector('iframe') as HTMLIFrameElement;

describe('design studio page', () => {
	it('says so when there is nothing to look at yet', () => {
		const root = studio([]);
		expect(text(root)).toContain('No previews yet');
		expect(frame(root)).toBeNull();
	});

	it('shows the newest version in a frame that has an origin of its own', () => {
		const root = studio([preview(1), preview(2)]);
		const shown = frame(root);
		expect(shown.getAttribute('src')).toBe(`/design-preview/${TOKEN}/index.html`);
		expect(shown.getAttribute('sandbox')).toBe('allow-scripts');
		expect(shown.getAttribute('referrerpolicy')).toBe('no-referrer');
		expect(text(root)).toContain('Editorial pass 2');
		expect(text(root)).toContain('Version 1');
		expect(root.querySelector('[aria-current="true"]')?.textContent).toContain('Version 2');
	});

	it('opens the version the link asked for, and switches when another is picked', async () => {
		const root = studio([preview(1), preview(2, { preview_token: 'cd'.repeat(32) })], '1');
		expect(frame(root).getAttribute('src')).toBe(`/design-preview/${TOKEN}/index.html`);
		const second = [...root.querySelectorAll('button')].find((b) => b.textContent?.includes('Version 2')) as HTMLButtonElement;
		second.click();
		await settle();
		expect(frame(root).getAttribute('src')).toBe(`/design-preview/${'cd'.repeat(32)}/index.html`);
	});

	it('sizes the frame for a phone and back', async () => {
		const root = studio([preview(1)]);
		const phone = root.querySelector('button[aria-label^="Phone"]') as HTMLButtonElement;
		phone.click();
		await settle();
		expect(frame(root).style.width).toBe('390px');
		(root.querySelector('button[aria-label="Desktop"]') as HTMLButtonElement).click();
		await settle();
		expect(frame(root).style.width).toBe('100%');
	});

	it('lists the page’s elements and shows the one picked from the list', async () => {
		const root = studio([preview(1)]);
		expect(text(root)).toContain('Nothing selected yet.');
		const hero = [...root.querySelectorAll('button')].find((b) => b.textContent?.includes('Hero')) as HTMLButtonElement;
		hero.click();
		await settle();
		expect(text(root)).not.toContain('Nothing selected yet.');
		expect(root.querySelector('.studio__selected')?.textContent).toContain('hero');
	});

	it('ignores a message that is not from its own frame', async () => {
		const root = studio([preview(1)]);
		window.dispatchEvent(new MessageEvent('message', { data: { source: PREVIEW_SOURCE, type: 'select', page: 'index.html', id: 'hero', tag: 'h1', text: 'Hi' }, source: window }));
		await settle();
		expect(text(root)).toContain('Nothing selected yet.');
	});

	it('links the published address when the version has one', () => {
		const root = studio([preview(1, { published_url: 'https://ship.example.com/dpl/d1' })]);
		const link = root.querySelector('a[href="https://ship.example.com/dpl/d1"]') as HTMLAnchorElement;
		expect(link.getAttribute('rel')).toContain('noopener');
	});
});
