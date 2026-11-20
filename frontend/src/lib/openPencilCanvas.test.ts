import { describe, expect, it } from 'vitest';
import {
	buildStudioSceneModel,
	exportStudioSceneToSVG,
	getStudioTargetBounds,
	hitTestStudioScene,
	paintOpenPencilSceneCanvas
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

	it('merges direction tokens into rawPenJson and spaces stacked frames horizontally', () => {
		const rawPen = JSON.stringify({
			version: '1.0',
			children: [
				{
					type: 'frame',
					id: 'screen-home',
					name: 'Home Screen',
					width: 1280,
					height: 800,
					fill: '$color-bg',
					children: [
						{
							type: 'frame',
							id: 'header',
							name: 'Site Header',
							width: 1200,
							height: 60,
							fill: '$color-surface',
							children: [
								{ type: 'text', id: 'title', content: 'Fresh Bakery', fill: '$color-text' }
							]
						}
					]
				},
				{
					type: 'frame',
					id: 'screen-menu',
					name: 'Menu Screen',
					width: 1280,
					height: 800,
					fill: '$color-bg',
					children: [
						{ type: 'text', id: 'menu-title', content: 'Menu Items', fill: '$color-text' }
					]
				}
			]
		});

		const model = buildStudioSceneModel({
			directions,
			preview,
			activeDirection: directions[0],
			device: 'desktop',
			rawPenJson: rawPen
		});

		expect(model.frames).toHaveLength(2);
		// Frames should not be stacked at x = 0
		expect(model.frames[0].x).toBe(0);
		expect(model.frames[1].x).toBeGreaterThan(1280);

		// Variables should have been merged
		const bgVar = model.variables.find((v) => v.name === 'color-bg');
		expect(bgVar).toBeDefined();
		expect(bgVar?.cssValue).toBe('rgb(250, 248, 245)');

		// Target bounds should reflect the spaced position
		const menuBounds = getStudioTargetBounds(model, 'menu/index.html', 'screen-menu');
		expect(menuBounds?.x).toBe(model.frames[1].x);
	});

	it('paints scene canvas with fills, strokes and text', () => {
		const rawPen = JSON.stringify({
			version: '1.0',
			children: [
				{
					type: 'frame',
					id: 'screen-home',
					name: 'Cash Flow Dashboard',
					width: 1280,
					height: 800,
					children: [
						{
							type: 'frame',
							id: 'header',
							name: 'Header',
							width: 1200,
							height: 60,
							fill: '#F1ECE4',
							stroke: { fill: '#DCD5CA', thickness: 1 },
							children: [
								{ type: 'text', id: 'title', content: 'Cash Flow', fill: '#1A1917' }
							]
						}
					]
				}
			]
		});

		const model = buildStudioSceneModel({
			directions,
			preview,
			activeDirection: directions[0],
			device: 'desktop',
			rawPenJson: rawPen
		});

		const calls: any[] = [];
		const dummyCtx: any = {
			measureText: (text: string) => ({ width: text.length * 8 }),
			save: () => calls.push({ method: 'save' }),
			restore: () => calls.push({ method: 'restore' }),
			scale: (...args: any[]) => calls.push({ method: 'scale', args }),
			translate: (...args: any[]) => calls.push({ method: 'translate', args }),
			clearRect: (...args: any[]) => calls.push({ method: 'clearRect', args }),
			beginPath: () => calls.push({ method: 'beginPath' }),
			rect: (...args: any[]) => calls.push({ method: 'rect', args }),
			roundRect: (...args: any[]) => calls.push({ method: 'roundRect', args }),
			fill: () => calls.push({ method: 'fill', fillStyle: dummyCtx.fillStyle }),
			stroke: () => calls.push({ method: 'stroke', strokeStyle: dummyCtx.strokeStyle, lineWidth: dummyCtx.lineWidth }),
			fillText: (...args: any[]) => calls.push({ method: 'fillText', args, fillStyle: dummyCtx.fillStyle }),
			fillRect: (...args: any[]) => calls.push({ method: 'fillRect', args }),
			strokeRect: (...args: any[]) => calls.push({ method: 'strokeRect', args }),
			clip: () => calls.push({ method: 'clip' })
		};

		const canvas: any = {
			clientWidth: 1200,
			clientHeight: 800,
			getContext: () => dummyCtx
		};

		paintOpenPencilSceneCanvas(canvas, model, {
			panX: 0,
			panY: 0,
			zoom: 1,
			hoveredNodeId: null,
			selectedNodeId: 'title'
		});

		// Verify fill operations
		const fills = calls.filter((c) => c.method === 'fill');
		expect(fills.length).toBeGreaterThan(0);
		// Verify text drawing
		const textCalls = calls.filter((c) => c.method === 'fillText');
		expect(textCalls.length).toBeGreaterThan(0);
		expect(textCalls[0].args[0]).toBe('Cash Flow');
	});

	it('normalizes author text properties (text, label, value) and card types into visible elements', () => {
		const rawPen = JSON.stringify({
			version: '1.0',
			variables: {
				'color-custom': '#123456'
			},
			children: [
				{
					type: 'frame',
					id: 'screen-dashboard',
					name: 'Cash Flow Dashboard',
					width: 1280,
					height: 800,
					children: [
						{
							type: 'card',
							id: 'summary-card',
							name: 'Summary Card',
							width: 400,
							height: 120,
							cornerRadius: 12,
							children: [
								{ type: 'label', id: 'lbl', text: 'Net Cash Flow', fill: '#FFFFFF' },
								{ type: 'text', id: 'val', value: 'Rp 45.000.000' }
							]
						},
						{
							type: 'component',
							id: 'action-btn',
							name: 'Action Button',
							width: 120,
							height: 40,
							children: [
								{ type: 'text', id: 'btn-txt', label: 'Submit' }
							]
						}
					]
				}
			]
		});

		const model = buildStudioSceneModel({
			directions,
			preview,
			activeDirection: directions[0],
			device: 'desktop',
			rawPenJson: rawPen
		});

		// Check that text nodes received content
		const lblNode = model.graph.getNode('lbl');
		expect(lblNode).toBeDefined();
		expect(lblNode?.text).toBe('Net Cash Flow');

		const valNode = model.graph.getNode('val');
		expect(valNode).toBeDefined();
		expect(valNode?.text).toBe('Rp 45.000.000');

		const btnNode = model.graph.getNode('action-btn');
		expect(btnNode?.type).toBe('COMPONENT');

		// Check canvas painting paints high-contrast text and card surfaces
		const calls: any[] = [];
		const dummyCtx: any = {
			measureText: (text: string) => ({ width: text.length * 8 }),
			save: () => calls.push({ method: 'save' }),
			restore: () => calls.push({ method: 'restore' }),
			scale: (...args: any[]) => calls.push({ method: 'scale', args }),
			translate: (...args: any[]) => calls.push({ method: 'translate', args }),
			clearRect: (...args: any[]) => calls.push({ method: 'clearRect', args }),
			beginPath: () => calls.push({ method: 'beginPath' }),
			rect: (...args: any[]) => calls.push({ method: 'rect', args }),
			roundRect: (...args: any[]) => calls.push({ method: 'roundRect', args }),
			fill: () => calls.push({ method: 'fill', fillStyle: dummyCtx.fillStyle }),
			stroke: () => calls.push({ method: 'stroke', strokeStyle: dummyCtx.strokeStyle, lineWidth: dummyCtx.lineWidth }),
			fillText: (...args: any[]) => calls.push({ method: 'fillText', args, fillStyle: dummyCtx.fillStyle }),
			fillRect: (...args: any[]) => calls.push({ method: 'fillRect', args }),
			strokeRect: (...args: any[]) => calls.push({ method: 'strokeRect', args }),
			clip: () => calls.push({ method: 'clip' })
		};

		const canvas: any = {
			clientWidth: 1280,
			clientHeight: 800,
			getContext: () => dummyCtx
		};

		paintOpenPencilSceneCanvas(canvas, model, {
			panX: 0,
			panY: 0,
			zoom: 1,
			hoveredNodeId: null,
			selectedNodeId: 'lbl'
		});

		const textCalls = calls.filter((c) => c.method === 'fillText');
		const drawnTexts = textCalls.map((c) => c.args[0]);
		expect(drawnTexts).toContain('Net Cash Flow');
		expect(drawnTexts).toContain('Rp 45.000.000');
		// White text on light artboard was normalized to high-contrast ink
		const lblCall = textCalls.find((c) => c.args[0] === 'Net Cash Flow');
		expect(lblCall?.fillStyle).toBe('#1A1917');
	});
});
