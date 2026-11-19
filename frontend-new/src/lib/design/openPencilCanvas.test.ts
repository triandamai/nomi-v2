import { describe, expect, it } from 'vitest';
import {
	buildStudioSceneModel,
	exportStudioSceneToSVG,
	getStudioTargetBounds,
	hitTestStudioScene
} from './openPencilCanvas';
import type { StudioDirection, StudioPreview } from './designStudio';

const directions: StudioDirection[] = [
	{
		id: 'editorial',
		name: 'Editorial Paper',
		summary: 'Warm paper tones, serif display headings, generous whitespace',
		rationale: 'Feels like an artisanal morning bakery menu',
		trade_offs: ['Less dense than a utility grid'],
		palette: [
			{ name: 'background', hex: '#FAF8F5' },
			{ name: 'surface', hex: '#F1ECE4' },
			{ name: 'text', hex: '#1A1917' },
			{ name: 'accent', hex: '#C85A32' }
		],
		typography: {
			heading: 'Instrument Serif',
			body: 'Inter'
		},
		layout: 'Single-column hero followed by two-column menu cards',
		recommended: true
	},
	{
		id: 'swiss',
		name: 'Swiss Grid',
		summary: 'High-contrast rules, bold grotesk type',
		rationale: 'Crisp and scannable on mobile',
		trade_offs: ['Cooler tone'],
		palette: [
			{ name: 'background', hex: '#FFFFFF' },
			{ name: 'text', hex: '#111111' },
			{ name: 'accent', hex: '#E11D48' }
		],
		typography: {
			heading: 'Inter',
			body: 'Inter'
		},
		layout: '12-column exposed border grid',
		recommended: false
	}
];

const preview: StudioPreview = {
	version: 1,
	direction_id: 'editorial',
	title: 'Roti Pagi — Editorial v1',
	pages: [
		{ path: 'index.html', title: 'Roti Pagi' },
		{ path: 'menu/index.html', title: 'Menu' }
	],
	elements: [
		{
			id: 'site-header',
			kind: 'component',
			label: 'Site header',
			pages: ['index.html', 'menu/index.html']
		},
		{
			id: 'hero',
			kind: 'section',
			label: 'Hero section',
			pages: ['index.html']
		},
		{
			id: 'menu-list',
			kind: 'section',
			label: 'Menu list',
			pages: ['menu/index.html']
		}
	],
	preview_token: 'tok-123',
	published_url: null,
	publish_error: null,
	created_at: '2026-11-18T00:00:00Z'
};

describe('OpenPencil Scene-Graph Canvas Engine', () => {
	it('Stage 1: builds an OpenPencil SceneGraph with style-tile artboards and hit-testable CTAs', () => {
		const model = buildStudioSceneModel({
			directions,
			preview: null,
			activeDirection: directions[0],
			device: 'desktop'
		});

		expect(model.frames).toHaveLength(2);
		expect(model.frames[0].kind).toBe('direction');
		expect(model.frames[0].directionId).toBe('editorial');
		expect(model.frames[1].directionId).toBe('swiss');
		expect(model.variables.some((v) => v.name === 'color-accent')).toBe(true);

		const firstFrame = model.frames[0];
		const hit = hitTestStudioScene(
			model,
			firstFrame.x + firstFrame.width / 2,
			firstFrame.y + firstFrame.height / 2
		);
		expect(hit).not.toBeNull();
		expect(hit?.target.directionId).toBe('editorial');
	});

	it('Stage 2 & 3: builds Design System + Multi-Page App Flow frames on the OpenPencil SceneGraph', () => {
		const model = buildStudioSceneModel({
			directions,
			preview,
			activeDirection: directions[0],
			device: 'desktop'
		});

		// 1 design-system frame + 2 page frames
		expect(model.frames).toHaveLength(3);
		expect(model.frames[0].kind).toBe('design-system');
		expect(model.frames[1].kind).toBe('page');
		expect(model.frames[1].pagePath).toBe('index.html');
		expect(model.frames[2].kind).toBe('page');
		expect(model.frames[2].pagePath).toBe('menu/index.html');

		const heroBounds = getStudioTargetBounds(model, 'index.html', 'hero');
		expect(heroBounds).not.toBeNull();

		const hitHero = hitTestStudioScene(
			model,
			heroBounds!.x + heroBounds!.width * 0.25,
			heroBounds!.y + heroBounds!.height * 0.5
		);
		expect(hitHero?.target.elementId).toBe('hero');
		expect(hitHero?.target.pagePath).toBe('index.html');
		expect(hitHero?.target.kind).toBe('section');
		expect(hitHero?.pin.x).toBeCloseTo(0.25, 1);

		const svg = exportStudioSceneToSVG(model, [model.frames[1].nodeId]);
		expect(svg).toContain('<svg');
	});

	it('parses raw .pen JSON documents directly when provided by Sora', () => {
		const rawPen = JSON.stringify({
			version: '1.0',
			variables: {
				'brand-accent': { type: 'color', value: '#C85A32' }
			},
			children: [
				{
					type: 'frame',
					id: 'home-frame',
					name: 'Roti Pagi',
					x: 0,
					y: 0,
					width: 1280,
					height: 800,
					fill: '#FAF8F5',
					layout: 'vertical',
					padding: 24,
					gap: 16,
					children: [
						{
							type: 'frame',
							id: 'hero',
							name: 'Hero section',
							width: 1200,
							height: 240,
							fill: '#F1ECE4'
						}
					]
				}
			]
		});

		const model = buildStudioSceneModel({
			directions,
			preview: {
				...preview,
				pages: [{ path: 'index.html', title: 'Roti Pagi' }]
			},
			activeDirection: directions[0],
			device: 'desktop',
			rawPenJson: rawPen
		});

		expect(model.frames).toHaveLength(1);
		const heroBounds = getStudioTargetBounds(model, 'index.html', 'hero');
		expect(heroBounds).not.toBeNull();
		expect(heroBounds?.width).toBe(1200);
	});
});
