import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { describe, expect, it, vi } from 'vitest';
import DesignCanvas from './DesignCanvas.svelte';
import { PREVIEW_SOURCE, type StudioDirectionSet, type StudioFeedback, type StudioPreview } from '$lib/design/designStudio';

const TOKEN = 'ab'.repeat(32);

const directions: StudioDirectionSet = {
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
			palette: [{ name: 'Surface', hex: '#FFFFFF' }],
			typography: { heading: 'Space Grotesk', body: 'IBM Plex Sans' },
			layout: 'Two-column menu table',
		},
	],
};

const previews: StudioPreview[] = [
	{
		version: 1,
		direction_id: 'editorial',
		title: 'Editorial',
		pages: [{ path: 'index.html', title: 'Home' }],
		elements: [{ id: 'hero', kind: 'section', label: 'Hero', pages: ['index.html'] }],
		preview_token: 'cd'.repeat(32),
		published_url: null,
		created_at: '2026-11-14T10:00:00Z',
	},
	{
		version: 2,
		direction_id: 'swiss',
		title: 'Swiss grid',
		pages: [
			{ path: 'index.html', title: 'Home' },
			{ path: 'menu/index.html', title: 'Menu' },
		],
		elements: [
			{ id: 'site-header', kind: 'component', label: 'Site header', pages: ['index.html', 'menu/index.html'] },
			{ id: 'hero', kind: 'section', label: 'Hero', pages: ['index.html'] },
			{ id: 'menu-grid', kind: 'section', label: 'Menu grid', pages: ['menu/index.html'] },
		],
		preview_token: TOKEN,
		published_url: 'https://preview.example/v2/',
		created_at: '2026-11-15T10:00:00Z',
	},
];

const openNote: StudioFeedback = {
	id: 'f1',
	number: 1,
	version: 2,
	level: 'section',
	page: 'index.html',
	element_id: 'hero',
	body: 'Make the headline shorter',
	pin: { x: 0.4, y: 0.3 },
	status: 'open',
	resolved_in_version: null,
	resolution: null,
	created_at: '2026-11-15T10:05:00Z',
	resolved_at: null,
};

describe('DesignCanvas', () => {
	it('draws the proposed directions on the vector canvas before any preview exists, with the canvas tools and no board tabs', async () => {
		render(DesignCanvas, { sessionId: 's1', directions, previews: [], feedback: [], approval: null });

		await expect.element(page.getByTestId('openpencil-canvas')).toBeInTheDocument();
		await expect.element(page.getByRole('toolbar', { name: 'Canvas tools' })).toBeInTheDocument();
		expect(document.body.textContent).not.toContain('All boards');
		expect(document.body.textContent).not.toContain('Artboards');
	});

	it("shows an HTML version's pages as artboards, with zoom and no publishing", async () => {
		await page.viewport(1280, 800);
		render(DesignCanvas, { sessionId: 's1', directions, previews, feedback: [], approval: null });

		await expect.element(page.getByTestId('spatial-canvas-viewport')).toBeInTheDocument();
		expect([...document.querySelectorAll('[data-artboard-page] iframe')].map((f) => f.getAttribute('src'))).toEqual([
			`/design-preview/${TOKEN}/index.html`,
			`/design-preview/${TOKEN}/menu/index.html`,
		]);
		await expect.element(page.getByTestId('canvas-zoom-label')).toHaveTextContent('100%');
		await page.getByRole('button', { name: 'Zoom out' }).click();
		await expect.element(page.getByTestId('canvas-zoom-label')).toHaveTextContent('75%');
		expect(document.querySelector('a[href="https://preview.example/v2/"]')).toBeNull();
		expect(document.body.textContent).not.toContain('Publish again');
	});

	it('draws a .pen version from its own file and never frames its pages as HTML', async () => {
		const pen = JSON.stringify({ version: '1.0', children: [{ id: 'design-system', type: 'frame', name: 'Design system', width: 390, height: 600 }] });
		const fetchSpy = vi.fn(async () => new Response(pen, { status: 200 }));
		vi.stubGlobal('fetch', fetchSpy);
		try {
			const penVersion: StudioPreview = { ...previews[1], version: 3, pages: [{ path: 'design-system', title: 'Design system' }], elements: [], pen_files: ['canvas.pen'] };
			render(DesignCanvas, { sessionId: 's1', directions, previews: [penVersion], feedback: [], approval: null });

			await expect.element(page.getByTestId('openpencil-canvas')).toBeInTheDocument();
			await vi.waitFor(() => expect(fetchSpy).toHaveBeenCalledWith(`/design-preview/${TOKEN}/canvas.pen`));
			expect(document.querySelector('iframe')).toBeNull();
		} finally {
			vi.unstubAllGlobals();
		}
	});

	it('picks a tool by click or key, selects without a note, and pins a note with the comment tool', async () => {
		const fetchSpy = vi.fn(async () => new Response(JSON.stringify(openNote), { status: 201, headers: { 'content-type': 'application/json' } }));
		vi.stubGlobal('fetch', fetchSpy);
		try {
			render(DesignCanvas, { sessionId: 's1', directions, previews, feedback: [], approval: null });

			await expect.element(page.getByRole('button', { name: 'Select (V)' })).toHaveAttribute('aria-pressed', 'true');
			window.dispatchEvent(new KeyboardEvent('keydown', { key: 'h' }));
			await expect.element(page.getByRole('button', { name: 'Move (H)' })).toHaveAttribute('aria-pressed', 'true');
			window.dispatchEvent(new KeyboardEvent('keydown', { key: 'v' }));

			const frame = document.querySelector('iframe[title="Preview of Home"]') as HTMLIFrameElement;
			const pick = () =>
				window.dispatchEvent(
					new MessageEvent('message', {
						source: frame.contentWindow,
						data: { source: PREVIEW_SOURCE, type: 'select', page: 'index.html', id: 'hero', tag: 'section', text: 'Fresh bread', point: { x: 0.4, y: 0.3 } },
					}),
				);
			pick();
			await new Promise((r) => setTimeout(r, 50));
			expect(document.querySelector('[data-testid="direct-pin-popover"]')).toBeNull();

			await page.getByRole('button', { name: 'Comment (C)' }).click();
			pick();
			await expect.element(page.getByTestId('direct-pin-popover')).toBeInTheDocument();
			await page.getByTestId('direct-pin-popover').getByPlaceholder('What should change?').fill('Make the headline shorter');
			await page.getByRole('button', { name: 'Save note' }).click();

			expect(fetchSpy).toHaveBeenCalledWith('/chat/s1/api/design/feedback', expect.objectContaining({ method: 'POST' }));
			await expect.element(page.getByTestId('revise-with-sora-bar')).toBeInTheDocument();
			await expect.element(page.getByRole('button', { name: 'Revise with Sora' })).toBeInTheDocument();
		} finally {
			vi.unstubAllGlobals();
		}
	});
});
