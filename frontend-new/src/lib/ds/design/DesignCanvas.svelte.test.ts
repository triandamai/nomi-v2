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
	it('renders Stage 1 direction style-tile artboards before any HTML preview exists and lets the person pick or adjust one', async () => {
		const fetchSpy = vi.fn(async () => new Response(JSON.stringify({ ok: true }), { status: 202, headers: { 'content-type': 'application/json' } }));
		vi.stubGlobal('fetch', fetchSpy);
		try {
			render(DesignCanvas, {
				sessionId: 's1',
				directions,
				previews: [],
				feedback: [],
				approval: null,
			});

			await expect.element(page.getByTestId('direction-board-editorial')).toBeInTheDocument();
			await expect.element(page.getByTestId('direction-board-swiss')).toBeInTheDocument();
			await expect.element(page.getByText('Recommended')).toBeInTheDocument();

			await page.getByTestId('direction-board-editorial').getByRole('button', { name: 'Use this direction' }).click();
			expect(fetchSpy).toHaveBeenCalledWith(
				'/chat/s1/api/design/refine',
				expect.objectContaining({
					method: 'POST',
					body: JSON.stringify({
						text: '@Sora Use the "Editorial" direction (editorial) and build the design system and multi-page preview.',
					}),
				}),
			);
		} finally {
			vi.unstubAllGlobals();
		}
	});

	it('renders the 2D Spatial Canvas with zoom controls, design system board, and multi-page app flow artboards', async () => {
		render(DesignCanvas, {
			sessionId: 's1',
			directions,
			previews,
			feedback: [],
			approval: null,
		});

		await expect.element(page.getByTestId('spatial-canvas-viewport')).toBeInTheDocument();
		await expect.element(page.getByTestId('design-system-artboard')).toBeInTheDocument();
		await expect.element(page.getByTestId('flow-artboard-index.html')).toBeInTheDocument();
		await expect.element(page.getByTestId('flow-artboard-menu/index.html')).toBeInTheDocument();
		await expect.element(page.getByTestId('canvas-zoom-label')).toHaveTextContent('100%');

		await page.getByRole('button', { name: 'Zoom out' }).click();
		await expect.element(page.getByTestId('canvas-zoom-label')).toHaveTextContent('75%');
	});

	it('opens the direct click-to-pin popover when an element is picked and queues open notes in the Revise with Sora bar', async () => {
		const fetchSpy = vi.fn(async () => new Response(JSON.stringify(openNote), { status: 201, headers: { 'content-type': 'application/json' } }));
		vi.stubGlobal('fetch', fetchSpy);
		try {
			render(DesignCanvas, {
				sessionId: 's1',
				directions,
				previews,
				feedback: [],
				approval: null,
			});

			const frame = document.querySelector('iframe[title="Preview of Home"]') as HTMLIFrameElement;
			window.dispatchEvent(
				new MessageEvent('message', {
					source: frame.contentWindow,
					data: { source: PREVIEW_SOURCE, type: 'select', page: 'index.html', id: 'hero', tag: 'section', text: 'Fresh bread', point: { x: 0.4, y: 0.3 } },
				}),
			);

			await expect.element(page.getByTestId('direct-pin-popover')).toBeInTheDocument();
			await page.getByTestId('direct-pin-popover').getByPlaceholder('What should change?').fill('Make the headline shorter');
			await page.getByRole('button', { name: 'Save note' }).click();

			expect(fetchSpy).toHaveBeenCalledWith(
				'/chat/s1/api/design/feedback',
				expect.objectContaining({ method: 'POST' }),
			);
			await expect.element(page.getByTestId('revise-with-sora-bar')).toBeInTheDocument();
			await expect.element(page.getByText('1 open notes')).toBeInTheDocument();
			await expect.element(page.getByRole('button', { name: 'Revise with Sora' })).toBeInTheDocument();
		} finally {
			vi.unstubAllGlobals();
		}
	});
});
