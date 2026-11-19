import {
	computeAllLayouts,
	createEditor,
	createLinter,
	recommended,
	renderNodesToSVG,
	type Editor
} from '@open-pencil/core';
import { parsePenFile, type PenDocument, type PenNode, type PenVariable } from '@open-pencil/pen';
import { colorToCSS, parseColor } from '@open-pencil/scene-graph/color';
import type { Color, SceneGraph, SceneNode } from '@open-pencil/scene-graph';
import {
	CANVAS_ARTBOARD_WIDTHS,
	type Device,
	type StudioDirection,
	type StudioElement,
	type StudioElementKind,
	type StudioPin,
	type StudioPreview
} from './designStudio';

export interface StudioCanvasFrame {
	id: string;
	nodeId: string;
	kind: 'direction' | 'design-system' | 'page';
	title: string;
	subtitle: string;
	pagePath: string | null;
	directionId: string | null;
	x: number;
	y: number;
	width: number;
	height: number;
}

export interface StudioNodeTarget {
	nodeId: string;
	elementId: string | null;
	pagePath: string | null;
	label: string;
	kind: StudioElementKind | 'page' | 'direction-cta';
	directionId?: string;
}

export interface StudioTokenVariable {
	id: string;
	name: string;
	type: 'COLOR' | 'FLOAT' | 'STRING' | 'BOOLEAN';
	cssValue: string;
}

export interface StudioLintIssue {
	ruleId: string;
	severity: 'error' | 'warning' | 'info';
	message: string;
	nodeId: string;
}

export interface StudioSceneModel {
	penDocument: PenDocument;
	penJson: string;
	graph: SceneGraph;
	editor: Editor;
	pageId: string;
	frames: StudioCanvasFrame[];
	nodeTargets: Map<string, StudioNodeTarget>;
	elementNodeIds: Map<string, string>;
	pageNodeIds: Map<string, string>;
	variables: StudioTokenVariable[];
	lintIssues: StudioLintIssue[];
}

export interface StudioHitResult {
	target: StudioNodeTarget;
	bounds: { x: number; y: number; width: number; height: number };
	pin: StudioPin;
}

function safeHex(raw: string | undefined, fallback: string): string {
	if (!raw) return fallback;
	const trimmed = raw.trim();
	if (/^#[0-9a-fA-F]{3,8}$/.test(trimmed)) return trimmed;
	return fallback;
}

function swatchLabel(swatch: { name?: string; role?: string } | undefined): string {
	if (!swatch) return 'color';
	return swatch.name ?? swatch.role ?? 'color';
}

function directionSummary(d: StudioDirection): string {
	return d.summary ?? (d as unknown as { looks_like?: string }).looks_like ?? '';
}

function directionRationale(d: StudioDirection): string {
	return d.rationale ?? (d as unknown as { why_it_fits?: string }).why_it_fits ?? '';
}

function directionTradeOffs(d: StudioDirection): string {
	if (Array.isArray(d.trade_offs)) return d.trade_offs.join('; ');
	return String(d.trade_offs ?? '');
}

function headingFont(d: StudioDirection): string {
	return (
		d.typography?.heading ??
		(d.typography as unknown as { heading_font?: string })?.heading_font ??
		'Instrument Serif'
	);
}

function bodyFont(d: StudioDirection): string {
	return (
		d.typography?.body ??
		(d.typography as unknown as { body_font?: string })?.body_font ??
		'Inter'
	);
}

function paletteColor(
	direction: StudioDirection | null,
	matcher: RegExp,
	fallbackIndex: number,
	fallbackHex: string
): string {
	if (!direction || !direction.palette || direction.palette.length === 0) return fallbackHex;
	const matched = direction.palette.find((entry) => matcher.test(swatchLabel(entry)));
	if (matched) return safeHex(matched.hex, fallbackHex);
	const byIdx = direction.palette[fallbackIndex];
	return byIdx ? safeHex(byIdx.hex, fallbackHex) : fallbackHex;
}

function slugifyId(raw: string): string {
	return (
		raw
			.toLowerCase()
			.replace(/[^a-z0-9]+/g, '-')
			.replace(/^-+|-+$/g, '') || 'node'
	);
}

/**
 * Builds an OpenPencil `.pen` document representing the current Design Studio state:
 * - Stage 1 (`!preview && directions.length > 0`): Style-Tile Artboard frames on the infinite canvas
 * - Stage 2 & 3 (`preview`): Design System & Variables frame + Multi-Page App Flow frames
 */
export function buildStudioPenDocument(options: {
	directions: StudioDirection[];
	preview: StudioPreview | null;
	activeDirection: StudioDirection | null;
	device?: Device;
}): {
	doc: PenDocument;
	frameMeta: Omit<StudioCanvasFrame, 'x' | 'y' | 'width' | 'height'>[];
	nodeTargets: Map<string, StudioNodeTarget>;
	elementNodeIds: Map<string, string>;
	pageNodeIds: Map<string, string>;
} {
	const { directions, preview, activeDirection, device = 'desktop' } = options;
	const variables: Record<string, PenVariable> = {};
	const frameMeta: Omit<StudioCanvasFrame, 'x' | 'y' | 'width' | 'height'>[] = [];
	const nodeTargets = new Map<string, StudioNodeTarget>();
	const elementNodeIds = new Map<string, string>();
	const pageNodeIds = new Map<string, string>();

	const dir = activeDirection ?? directions[0] ?? null;
	const bgHex = paletteColor(dir, /bg|background|canvas|paper/i, 0, '#FAF8F5');
	const surfaceHex = paletteColor(dir, /surface|card|panel|muted/i, 1, '#F1ECE4');
	const textHex = paletteColor(dir, /text|ink|fg|foreground|primary-text/i, 2, '#1A1917');
	const accentHex = paletteColor(dir, /accent|brand|primary|cta/i, 3, '#C85A32');
	const borderHex = paletteColor(dir, /border|line|divider|stroke/i, 1, '#DCD5CA');

	variables['color-bg'] = { type: 'color', value: bgHex };
	variables['color-surface'] = { type: 'color', value: surfaceHex };
	variables['color-text'] = { type: 'color', value: textHex };
	variables['color-accent'] = { type: 'color', value: accentHex };
	variables['color-border'] = { type: 'color', value: borderHex };

	if (dir) {
		for (const swatch of dir.palette ?? []) {
			const tokenName = `palette-${slugifyId(swatchLabel(swatch))}`;
			variables[tokenName] = { type: 'color', value: safeHex(swatch.hex, accentHex) };
		}
		variables['font-heading'] = {
			type: 'string',
			value: headingFont(dir)
		};
		variables['font-body'] = {
			type: 'string',
			value: bodyFont(dir)
		};
	}

	const children: PenNode[] = [];

	// Stage 1: Style Direction Exploration Artboards when no preview exists yet
	if (!preview && directions.length > 0) {
		const tileW = 460;
		const tileGap = 56;
		directions.forEach((d, idx) => {
			const frameId = `direction-frame-${slugifyId(d.id)}`;
			const dBg = paletteColor(d, /bg|background|canvas|paper/i, 0, '#FAF8F5');
			const dSurface = paletteColor(d, /surface|card|panel/i, 1, '#F1ECE4');
			const dText = paletteColor(d, /text|ink|fg|foreground/i, 2, '#1A1917');
			const dAccent = paletteColor(d, /accent|brand|primary/i, 3, '#C85A32');
			const ctaId = `pick-direction-${slugifyId(d.id)}`;
			const dHeading = headingFont(d);
			const dBody = bodyFont(d);

			const swatches: PenNode[] = (d.palette ?? []).slice(0, 6).map((swatch, sIdx) => ({
				type: 'frame',
				id: `${frameId}-swatch-${sIdx}`,
				name: `${swatchLabel(swatch)} (${swatch.hex})`,
				width: 64,
				height: 52,
				cornerRadius: 8,
				fill: safeHex(swatch.hex, dAccent),
				stroke: { align: 'inside', thickness: 1, fill: borderHex },
				layout: 'vertical',
				padding: 6,
				justifyContent: 'end',
				children: [
					{
						type: 'text',
						id: `${frameId}-swatch-hex-${sIdx}`,
						name: swatchLabel(swatch),
						content: swatch.hex,
						fontSize: 9,
						fontWeight: 600,
						fill: dText
					}
				]
			}));

			const tileNode: PenNode = {
				type: 'frame',
				id: frameId,
				name: d.recommended ? `${d.name} ★` : d.name,
				x: idx * (tileW + tileGap),
				y: 0,
				width: tileW,
				height: 580,
				fill: dBg,
				cornerRadius: 16,
				stroke: {
					align: 'inside',
					thickness: d.recommended ? 2 : 1,
					fill: d.recommended ? dAccent : borderHex
				},
				layout: 'vertical',
				padding: 28,
				gap: 18,
				children: [
					{
						type: 'frame',
						id: `${frameId}-header`,
						name: 'Direction Header',
						width: tileW - 56,
						height: 76,
						fill: dSurface,
						cornerRadius: 10,
						layout: 'vertical',
						padding: 14,
						gap: 6,
						children: [
							{
								type: 'text',
								id: `${frameId}-title`,
								name: d.name,
								content: d.recommended ? `${d.name} · Recommended` : d.name,
								fontFamily: dHeading,
								fontSize: 20,
								fontWeight: 700,
								fill: dText
							},
							{
								type: 'text',
								id: `${frameId}-summary`,
								name: 'Look & Feel',
								content: directionSummary(d),
								fontFamily: dBody,
								fontSize: 12,
								fill: dText
							}
						]
					},
					{
						type: 'frame',
						id: `${frameId}-palette`,
						name: 'Colour Palette',
						width: tileW - 56,
						height: 60,
						layout: 'horizontal',
						gap: 8,
						children: swatches
					},
					{
						type: 'frame',
						id: `${frameId}-type-card`,
						name: 'Typography Specimen',
						width: tileW - 56,
						height: 112,
						fill: dSurface,
						cornerRadius: 10,
						layout: 'vertical',
						padding: 14,
						gap: 6,
						children: [
							{
								type: 'text',
								id: `${frameId}-heading-font`,
								name: 'Heading Specimen',
								content: `Aa · ${dHeading}`,
								fontFamily: dHeading,
								fontSize: 22,
								fontWeight: 700,
								fill: dText
							},
							{
								type: 'text',
								id: `${frameId}-body-font`,
								name: 'Body Specimen',
								content: `Body: ${dBody}`,
								fontFamily: dBody,
								fontSize: 12,
								fill: dText
							},
							{
								type: 'text',
								id: `${frameId}-why`,
								name: 'Why it fits',
								content: directionRationale(d),
								fontSize: 11,
								fill: dText
							}
						]
					},
					{
						type: 'frame',
						id: `${frameId}-layout-card`,
						name: 'Layout & Trade-offs',
						width: tileW - 56,
						height: 124,
						fill: dSurface,
						cornerRadius: 10,
						layout: 'vertical',
						padding: 14,
						gap: 6,
						children: [
							{
								type: 'text',
								id: `${frameId}-layout-text`,
								name: 'Layout Structure',
								content: `Layout: ${d.layout}`,
								fontSize: 12,
								fontWeight: 600,
								fill: dText
							},
							{
								type: 'text',
								id: `${frameId}-tradeoffs`,
								name: 'Trade-offs',
								content: `Trade-offs: ${directionTradeOffs(d)}`,
								fontSize: 11,
								fill: dText
							}
						]
					},
					{
						type: 'frame',
						id: ctaId,
						name: `Pick ${d.name}`,
						width: tileW - 56,
						height: 44,
						fill: dAccent,
						cornerRadius: 10,
						layout: 'horizontal',
						justifyContent: 'center',
						alignItems: 'center',
						padding: 10,
						children: [
							{
								type: 'text',
								id: `${ctaId}-label`,
								name: 'CTA Label',
								content: `Pick ${d.name} → Build Design System`,
								fontSize: 13,
								fontWeight: 600,
								fill: '#FFFFFF'
							}
						]
					}
				]
			};

			children.push(tileNode);
			frameMeta.push({
				id: frameId,
				nodeId: frameId,
				kind: 'direction',
				title: d.name,
				subtitle: d.recommended ? 'Recommended direction' : 'Style direction',
				pagePath: null,
				directionId: d.id
			});
			nodeTargets.set(ctaId, {
				nodeId: ctaId,
				elementId: null,
				pagePath: null,
				label: `Pick ${d.name}`,
				kind: 'direction-cta',
				directionId: d.id
			});
			nodeTargets.set(frameId, {
				nodeId: frameId,
				elementId: null,
				pagePath: null,
				label: d.name,
				kind: 'direction-cta',
				directionId: d.id
			});
		});

		return {
			doc: { version: '1.0', variables, children },
			frameMeta,
			nodeTargets,
			elementNodeIds,
			pageNodeIds
		};
	}

	if (!preview) {
		return {
			doc: { version: '1.0', variables, children: [] },
			frameMeta,
			nodeTargets,
			elementNodeIds,
			pageNodeIds
		};
	}

	let cursorX = 0;
	const frameGap = 88;

	// Stage 2: Design System & Variable Tokens Frame on the left of the canvas
	const components = preview.elements.filter((el) => el.kind === 'component');
	if (dir || components.length > 0) {
		const dsWidth = 520;
		const dsFrameId = 'design-system-artboard';
		const swatchNodes: PenNode[] = (dir?.palette ?? []).slice(0, 6).map((swatch, idx) => {
			const label = swatchLabel(swatch);
			return {
				type: 'frame',
				id: `ds-token-swatch-${idx}`,
				name: `${label} (${swatch.hex})`,
				width: 72,
				height: 56,
				cornerRadius: 8,
				fill: safeHex(swatch.hex, accentHex),
				stroke: { align: 'inside', thickness: 1, fill: borderHex },
				layout: 'vertical',
				padding: 6,
				justifyContent: 'end',
				children: [
					{
						type: 'text',
						id: `ds-token-swatch-label-${idx}`,
						name: label,
						content: label,
						fontSize: 9,
						fontWeight: 600,
						fill: textHex
					}
				]
			};
		});

		const componentRows: PenNode[] = components.slice(0, 8).map((comp) => {
			const compNodeId = `ds-comp-${slugifyId(comp.id)}`;
			const firstPage = comp.pages[0] ?? preview.pages[0]?.path ?? 'index.html';
			nodeTargets.set(compNodeId, {
				nodeId: compNodeId,
				elementId: comp.id,
				pagePath: firstPage,
				label: comp.label,
				kind: 'component'
			});
			return {
				type: 'frame',
				id: compNodeId,
				name: comp.label,
				reusable: true,
				width: dsWidth - 48,
				height: 48,
				fill: surfaceHex,
				cornerRadius: 10,
				stroke: { align: 'inside', thickness: 1, fill: accentHex },
				layout: 'horizontal',
				padding: 12,
				justifyContent: 'space_between',
				alignItems: 'center',
				children: [
					{
						type: 'text',
						id: `${compNodeId}-title`,
						name: comp.label,
						content: `❖ ${comp.label}`,
						fontSize: 13,
						fontWeight: 600,
						fill: textHex
					},
					{
						type: 'text',
						id: `${compNodeId}-slug`,
						name: comp.id,
						content: `#${comp.id} · ${comp.pages.length}p`,
						fontSize: 11,
						fill: accentHex
					}
				]
			};
		});

		const dsHeight = Math.max(420, 220 + componentRows.length * 58);
		const dsNode: PenNode = {
			type: 'frame',
			id: dsFrameId,
			name: `Design System · ${dir?.name ?? preview.direction_id}`,
			x: cursorX,
			y: 0,
			width: dsWidth,
			height: dsHeight,
			fill: bgHex,
			cornerRadius: 16,
			stroke: { align: 'inside', thickness: 1, fill: borderHex },
			layout: 'vertical',
			padding: 24,
			gap: 16,
			children: [
				{
					type: 'frame',
					id: 'ds-header',
					name: 'Design System Header',
					width: dsWidth - 48,
					height: 60,
					fill: surfaceHex,
					cornerRadius: 10,
					layout: 'vertical',
					padding: 12,
					gap: 4,
					children: [
						{
							type: 'text',
							id: 'ds-title',
							name: 'System Title',
							content: `Design System · ${dir?.name ?? preview.direction_id}`,
							fontFamily: '$font-heading',
							fontSize: 18,
							fontWeight: 700,
							fill: '$color-text'
						},
						{
							type: 'text',
							id: 'ds-subtitle',
							name: 'Typography Tokens',
							content: dir
								? `${headingFont(dir)} / ${bodyFont(dir)}`
								: `${components.length} reusable components`,
							fontSize: 12,
							fill: '$color-text'
						}
					]
				},
				...(swatchNodes.length > 0
					? [
							{
								type: 'frame',
								id: 'ds-swatches-row',
								name: 'Variable Tokens',
								width: dsWidth - 48,
								height: 64,
								layout: 'horizontal',
								gap: 8,
								children: swatchNodes
							} satisfies PenNode
						]
					: []),
				...componentRows
			]
		};

		children.push(dsNode);
		frameMeta.push({
			id: dsFrameId,
			nodeId: dsFrameId,
			kind: 'design-system',
			title: `Design System · ${dir?.name ?? preview.direction_id}`,
			subtitle: 'Tokens & Reusable Components',
			pagePath: preview.pages[0]?.path ?? null,
			directionId: preview.direction_id
		});
		cursorX += dsWidth + frameGap;
	}

	// Stage 3: Multi-Page App Flow Artboards
	const artboardWidth = CANVAS_ARTBOARD_WIDTHS[device];
	const innerWidth = artboardWidth - 48;

	preview.pages.forEach((page, pageIdx) => {
		const pageFrameId = `page-frame-${pageIdx}-${slugifyId(page.path)}`;
		const pageElements: StudioElement[] = preview.elements.filter((el) =>
			el.pages.includes(page.path)
		);

		const sectionNodes: PenNode[] = pageElements.map((el, elIdx) => {
			const nodeId = `node-p${pageIdx}-${slugifyId(el.id)}`;
			const isComponent = el.kind === 'component';
			const isSection = el.kind === 'section';
			const boxHeight = isComponent ? 72 : isSection ? 148 : 88;

			elementNodeIds.set(`${page.path}::${el.id}`, nodeId);
			if (!elementNodeIds.has(el.id)) {
				elementNodeIds.set(el.id, nodeId);
			}
			nodeTargets.set(nodeId, {
				nodeId,
				elementId: el.id,
				pagePath: page.path,
				label: el.label,
				kind: el.kind
			});

			return {
				type: 'frame',
				id: nodeId,
				name: el.label,
				reusable: isComponent,
				width: innerWidth,
				height: boxHeight,
				fill: isComponent ? surfaceHex : isSection && elIdx === 0 ? surfaceHex : bgHex,
				cornerRadius: 12,
				stroke: {
					align: 'inside',
					thickness: isComponent ? 1.5 : 1,
					fill: isComponent ? accentHex : borderHex
				},
				layout: 'vertical',
				padding: 16,
				gap: 8,
				children: [
					{
						type: 'frame',
						id: `${nodeId}-bar`,
						name: `${el.label} Header`,
						width: innerWidth - 32,
						height: 24,
						layout: 'horizontal',
						justifyContent: 'space_between',
						alignItems: 'center',
						children: [
							{
								type: 'text',
								id: `${nodeId}-title`,
								name: el.label,
								content: isComponent ? `❖ ${el.label}` : el.label,
								fontFamily: isSection ? '$font-heading' : '$font-body',
								fontSize: isSection ? 18 : 14,
								fontWeight: 700,
								fill: '$color-text'
							},
							{
								type: 'text',
								id: `${nodeId}-badge`,
								name: el.id,
								content: `${el.kind} · #${el.id}`,
								fontSize: 11,
								fontWeight: 500,
								fill: isComponent ? '$color-accent' : '$color-text'
							}
						]
					},
					{
						type: 'text',
						id: `${nodeId}-desc`,
						name: 'Layer Summary',
						content:
							el.pages.length > 1
								? `Shared ${el.kind} across ${el.pages.length} pages (${el.pages.join(', ')}) — click to pin feedback`
								: `${el.kind === 'section' ? 'Page section' : 'Interactive layer'} on ${page.title} (${page.path}) — click anywhere to inspect or pin feedback`,
						fontFamily: '$font-body',
						fontSize: 12,
						fill: '$color-text'
					}
				]
			};
		});

		const frameHeight = Math.max(
			560,
			120 +
				sectionNodes.reduce(
					(sum, n) => sum + (typeof n.height === 'number' ? n.height : 110) + 16,
					0
				)
		);

		const pageFrame: PenNode = {
			type: 'frame',
			id: pageFrameId,
			name: `${page.title} (${page.path})`,
			x: cursorX,
			y: 0,
			width: artboardWidth,
			height: frameHeight,
			fill: '$color-bg',
			cornerRadius: 16,
			stroke: { align: 'inside', thickness: 1, fill: borderHex },
			layout: 'vertical',
			padding: 24,
			gap: 16,
			children: [
				{
					type: 'frame',
					id: `${pageFrameId}-chrome`,
					name: 'Artboard Header',
					width: innerWidth,
					height: 52,
					fill: '$color-surface',
					cornerRadius: 10,
					layout: 'horizontal',
					padding: 14,
					justifyContent: 'space_between',
					alignItems: 'center',
					children: [
						{
							type: 'text',
							id: `${pageFrameId}-title`,
							name: page.title,
							content: page.title,
							fontFamily: '$font-heading',
							fontSize: 16,
							fontWeight: 700,
							fill: '$color-text'
						},
						{
							type: 'text',
							id: `${pageFrameId}-path`,
							name: page.path,
							content: `${page.path} · ${artboardWidth}px`,
							fontSize: 11,
							fill: '$color-accent'
						}
					]
				},
				...sectionNodes
			]
		};

		children.push(pageFrame);
		pageNodeIds.set(page.path, pageFrameId);
		nodeTargets.set(pageFrameId, {
			nodeId: pageFrameId,
			elementId: null,
			pagePath: page.path,
			label: page.title,
			kind: 'page'
		});
		frameMeta.push({
			id: pageFrameId,
			nodeId: pageFrameId,
			kind: 'page',
			title: page.title,
			subtitle: page.path,
			pagePath: page.path,
			directionId: preview.direction_id
		});

		cursorX += artboardWidth + frameGap;
	});

	return {
		doc: { version: '1.0', variables, children },
		frameMeta,
		nodeTargets,
		elementNodeIds,
		pageNodeIds
	};
}

/**
 * The preview page a top-level `.pen` frame is: the one named by its id or its name, else the
 * one in its place. A version's pages come from its frames (design_artifact::parse_pen_document).
 */
function pageOfFrame(
	preview: StudioPreview | null,
	frame: { id: string; name?: string },
	index: number
): StudioPreview['pages'][number] | undefined {
	const pages = preview?.pages ?? [];
	return pages.find((p) => p.path === frame.id) ?? pages.find((p) => !!frame.name && p.title === frame.name) ?? pages[index];
}

/**
 * Builds and lays out a complete OpenPencil `SceneGraph` + headless `Editor` + linter pass
 * from either a raw `.pen` JSON file or the current Design Studio state.
 */
export function buildStudioSceneModel(options: {
	directions: StudioDirection[];
	preview: StudioPreview | null;
	activeDirection: StudioDirection | null;
	device?: Device;
	rawPenJson?: string | null;
}): StudioSceneModel {
	const built = buildStudioPenDocument(options);
	const penJson = options.rawPenJson?.trim() || JSON.stringify(built.doc);
	let graph: SceneGraph;
	let penDocument: PenDocument = built.doc;
	let normalizedPenJson = penJson;

	if (options.rawPenJson?.trim()) {
		try {
			const parsed = JSON.parse(options.rawPenJson) as Record<string, unknown>;
			// Merge default design tokens from built.doc.variables if not explicitly defined
			if (built.doc.variables) {
				const existingVars =
					parsed.variables && typeof parsed.variables === 'object' && !Array.isArray(parsed.variables)
						? (parsed.variables as Record<string, PenVariable>)
						: {};
				parsed.variables = { ...built.doc.variables, ...existingVars };
			}
			// Auto-arrange top-level frames if they lack x or are all stacked at 0
			if (Array.isArray(parsed.children)) {
				const frames = (parsed.children as Array<Record<string, unknown>>).filter(
					(c) => c && (c.type === 'frame' || !c.type)
				);
				const allAtZero = frames.length > 1 && frames.every((f) => !f.x || f.x === 0);
				if (allAtZero) {
					let cursorX = 0;
					const frameGap = 64;
					const defaultWidth = CANVAS_ARTBOARD_WIDTHS[options.device ?? 'desktop'];
					for (const frame of frames) {
						frame.x = cursorX;
						frame.y = (frame.y as number) ?? 0;
						const w = typeof frame.width === 'number' && frame.width > 0 ? frame.width : defaultWidth;
						cursorX += w + frameGap;
					}
				}
			}
			normalizedPenJson = JSON.stringify(parsed);
			penDocument = parsed as unknown as PenDocument;
			graph = parsePenFile(normalizedPenJson);
		} catch {
			graph = parsePenFile(JSON.stringify(built.doc));
		}
	} else {
		graph = parsePenFile(penJson);
	}

	const pages = graph.getPages(false);
	const pageId = pages[0]?.id ?? graph.rootId;
	computeAllLayouts(graph, pageId);

	const editor = createEditor({ graph });
	const nodeTargets = new Map(built.nodeTargets);
	const elementNodeIds = new Map(built.elementNodeIds);
	const pageNodeIds = new Map(built.pageNodeIds);

	// If loaded from rawPenJson, also index every top-level frame and child node in the parsed SceneGraph
	if (options.rawPenJson?.trim()) {
		const topFrames = graph.getChildren(pageId);
		topFrames.forEach((frame, idx) => {
			const pagePath = pageOfFrame(options.preview, frame, idx)?.path ?? frame.id;
			pageNodeIds.set(pagePath, frame.id);
			if (!nodeTargets.has(frame.id)) {
				nodeTargets.set(frame.id, {
					nodeId: frame.id,
					elementId: null,
					pagePath,
					label: frame.name || pagePath,
					kind: 'page'
				});
			}
			for (const entry of graph.flattenTree(frame.id)) {
				if (entry.node.id === frame.id) continue;
				const elMatch = options.preview?.elements.find((e) => e.id === entry.node.id);
				const kind: StudioElementKind =
					elMatch?.kind ??
					(entry.node.type === 'COMPONENT' || entry.node.type === 'INSTANCE'
						? 'component'
						: entry.depth <= 1
							? 'section'
							: 'element');
				elementNodeIds.set(`${pagePath}::${entry.node.id}`, entry.node.id);
				if (!elementNodeIds.has(entry.node.id)) {
					elementNodeIds.set(entry.node.id, entry.node.id);
				}
				nodeTargets.set(entry.node.id, {
					nodeId: entry.node.id,
					elementId: entry.node.id,
					pagePath,
					label: elMatch?.label ?? (entry.node.name || entry.node.id),
					kind
				});
			}
		});
	}

	const topFrames = graph.getChildren(pageId);
	const frames: StudioCanvasFrame[] = topFrames.map((node, index) => {
		const bounds = graph.getAbsoluteBounds(node.id);
		const meta = built.frameMeta.find((m) => m.nodeId === node.id);
		const fallbackPage = pageOfFrame(options.preview, node, index);
		return {
			id: meta?.id ?? node.id,
			nodeId: node.id,
			kind: meta?.kind ?? 'page',
			title: meta?.title ?? (node.name || fallbackPage?.title || `Frame ${index + 1}`),
			subtitle: meta?.subtitle ?? fallbackPage?.path ?? `${Math.round(bounds.width)}×${Math.round(bounds.height)}`,
			pagePath: meta?.pagePath ?? fallbackPage?.path ?? null,
			directionId: meta?.directionId ?? options.preview?.direction_id ?? null,
			x: bounds.x,
			y: bounds.y,
			width: bounds.width,
			height: bounds.height
		};
	});

	const variables: StudioTokenVariable[] = [];
	for (const variable of graph.variables.values()) {
		if (variable.type === 'COLOR') {
			const resolved = graph.resolveColorVariable(variable.id);
			variables.push({
				id: variable.id,
				name: variable.name,
				type: 'COLOR',
				cssValue: resolved ? colorToCSS(resolved) : '#000000'
			});
		} else {
			const val = graph.resolveVariable(variable.id);
			variables.push({
				id: variable.id,
				name: variable.name,
				type: variable.type,
				cssValue: String(val ?? '')
			});
		}
	}

	let lintIssues: StudioLintIssue[] = [];
	try {
		const linter = createLinter({ preset: 'recommended' });
		const lintResult = linter.lintGraph(graph);
		lintIssues = lintResult.messages.slice(0, 25).map((m) => ({
			ruleId: m.ruleId,
			severity: m.severity,
			message: m.message,
			nodeId: m.nodeId
		}));
	} catch {
		lintIssues = [];
	}

	return {
		penDocument,
		penJson: normalizedPenJson,
		graph,
		editor,
		pageId,
		frames,
		nodeTargets,
		elementNodeIds,
		pageNodeIds,
		variables,
		lintIssues
	};
}

/**
 * Hit-tests a point `(canvasX, canvasY)` in OpenPencil world coordinates and returns
 * the semantic target (`section`, `component`, `element`, `page`, or `direction-cta`),
 * its absolute bounding box, and the normalized pin `(0..1)`.
 */
export function hitTestStudioScene(
	model: StudioSceneModel,
	canvasX: number,
	canvasY: number
): StudioHitResult | null {
	const rawHit =
		model.graph.hitTestDeep(canvasX, canvasY, model.pageId, 2) ??
		model.editor.hitTestAtPoint(canvasX, canvasY, true);
	if (!rawHit) return null;

	const matchedNode = model.graph.closest(rawHit.id, (n) => model.nodeTargets.has(n.id));
	if (!matchedNode) return null;

	const target = model.nodeTargets.get(matchedNode.id);
	if (!target) return null;

	const bounds = model.graph.getAbsoluteBounds(matchedNode.id);
	const relX = bounds.width > 0 ? (canvasX - bounds.x) / bounds.width : 0.5;
	const relY = bounds.height > 0 ? (canvasY - bounds.y) / bounds.height : 0.5;
	const pin: StudioPin = {
		x: Math.round(Math.min(1, Math.max(0, relX)) * 1000) / 1000,
		y: Math.round(Math.min(1, Math.max(0, relY)) * 1000) / 1000
	};

	return {
		target,
		bounds,
		pin
	};
}

/**
 * Looks up the OpenPencil scene-graph bounding box for an element (`elementId`) on `pagePath`,
 * or for the page frame itself when `elementId` is null.
 */
export function getStudioTargetBounds(
	model: StudioSceneModel,
	pagePath: string | null,
	elementId: string | null
): { x: number; y: number; width: number; height: number; nodeId: string } | null {
	if (elementId) {
		const scopedId = pagePath ? model.elementNodeIds.get(`${pagePath}::${elementId}`) : undefined;
		const nodeId = scopedId ?? model.elementNodeIds.get(elementId);
		if (nodeId && model.graph.getNode(nodeId)) {
			const b = model.graph.getAbsoluteBounds(nodeId);
			return { ...b, nodeId };
		}
	}
	if (pagePath) {
		const pageNodeId = model.pageNodeIds.get(pagePath);
		if (pageNodeId && model.graph.getNode(pageNodeId)) {
			const b = model.graph.getAbsoluteBounds(pageNodeId);
			return { ...b, nodeId: pageNodeId };
		}
	}
	const firstFrame = model.frames[0];
	if (firstFrame) {
		return {
			x: firstFrame.x,
			y: firstFrame.y,
			width: firstFrame.width,
			height: firstFrame.height,
			nodeId: firstFrame.nodeId
		};
	}
	return null;
}

/**
 * Exports a specific frame or the entire current OpenPencil page to vector SVG using `@open-pencil/core`.
 */
export function exportStudioSceneToSVG(model: StudioSceneModel, nodeIds?: string[]): string | null {
	const ids = nodeIds && nodeIds.length > 0 ? nodeIds : model.frames.map((f) => f.nodeId);
	if (ids.length === 0) return null;
	return renderNodesToSVG(model.graph, model.pageId, ids, { xmlDeclaration: false });
}

function resolveNodeFillCss(graph: SceneGraph, node: SceneNode): string | null {
	for (let i = 0; i < node.fills.length; i++) {
		const fill = node.fills[i];
		if (!fill || fill.visible === false || fill.type !== 'SOLID') continue;
		const boundVarId =
			node.boundVariables[`fills[${i}]`] ??
			node.boundVariables[`fills[${i}].color`] ??
			node.boundVariables['fill'];
		let color: Color = fill.color;
		if (boundVarId) {
			const resolved = graph.resolveColorVariable(boundVarId);
			if (resolved) color = resolved;
		}
		return colorToCSS({ ...color, a: color.a * (fill.opacity ?? 1) });
	}
	return null;
}

function resolveNodeStrokeCss(
	graph: SceneGraph,
	node: SceneNode
): { css: string; weight: number } | null {
	for (let i = 0; i < node.strokes.length; i++) {
		const stroke = node.strokes[i];
		if (!stroke || stroke.visible === false) continue;
		const boundVarId =
			node.boundVariables[`strokes[${i}]`] ??
			node.boundVariables[`strokes[${i}].color`] ??
			node.boundVariables['stroke'];
		let color: Color = stroke.color;
		if (boundVarId) {
			const resolved = graph.resolveColorVariable(boundVarId);
			if (resolved) color = resolved;
		}
		return {
			css: colorToCSS({ ...color, a: color.a * (stroke.opacity ?? 1) }),
			weight: stroke.weight || 1
		};
	}
	return null;
}

function roundedRectPath(
	ctx: CanvasRenderingContext2D,
	x: number,
	y: number,
	w: number,
	h: number,
	r: number
): void {
	const radius = Math.max(0, Math.min(r, Math.min(w, h) / 2));
	ctx.beginPath();
	if (typeof ctx.roundRect === 'function') {
		ctx.roundRect(x, y, w, h, radius);
	} else {
		ctx.rect(x, y, w, h);
	}
}

/**
 * Paints the OpenPencil `SceneGraph` onto an HTML `<canvas>` at 60fps with DPI scaling,
 * selection/hover chrome, component badges, and rulers, and keeps `model.editor.state` synchronized.
 */
export function paintOpenPencilSceneCanvas(
	canvas: HTMLCanvasElement,
	model: StudioSceneModel,
	view: {
		panX: number;
		panY: number;
		zoom: number;
		hoveredNodeId: string | null;
		selectedNodeId: string | null;
		canvasBgCss?: string;
		accentCss?: string;
		componentCss?: string;
	}
): void {
	const dpr = typeof window !== 'undefined' ? window.devicePixelRatio || 1 : 1;
	const cssW = canvas.clientWidth || 960;
	const cssH = canvas.clientHeight || 640;
	const targetW = Math.max(1, Math.round(cssW * dpr));
	const targetH = Math.max(1, Math.round(cssH * dpr));
	if (canvas.width !== targetW || canvas.height !== targetH) {
		canvas.width = targetW;
		canvas.height = targetH;
	}

	// Keep OpenPencil's EditorState synchronized with the viewport & selection
	model.editor.state.panX = view.panX;
	model.editor.state.panY = view.panY;
	model.editor.state.zoom = view.zoom;
	model.editor.setHoveredNode(view.hoveredNodeId);
	if (view.selectedNodeId) {
		model.editor.select([view.selectedNodeId]);
	} else {
		model.editor.clearSelection();
	}

	const rawCtx = canvas.getContext('2d');
	if (!rawCtx) return;
	const ctx: CanvasRenderingContext2D = rawCtx;

	ctx.save();
	ctx.scale(dpr, dpr);
	ctx.clearRect(0, 0, cssW, cssH);

	// 1. Transform into OpenPencil world coordinates
	ctx.save();
	ctx.translate(view.panX, view.panY);
	ctx.scale(view.zoom, view.zoom);

	const accentCss = view.accentCss || '#2563EB';
	const componentCss = view.componentCss || '#8B5CF6';

	const topNodes = model.graph.getChildren(model.pageId);
	const topNodeIds = new Set(topNodes.map((n) => n.id));

	function drawSceneSubtree(node: SceneNode, isTopLevel = false): void {
		if (!node.visible) return;
		const bounds = model.graph.getAbsoluteBounds(node.id);
		let fillCss = resolveNodeFillCss(model.graph, node);
		let stroke = resolveNodeStrokeCss(model.graph, node);
		const radius = node.cornerRadius || 0;

		ctx.save();
		if (node.opacity < 1) {
			ctx.globalAlpha *= node.opacity;
		}

		let filled = false;
		if (isTopLevel && (node.type === 'FRAME' || node.type === 'COMPONENT')) {
			if (!fillCss) {
				fillCss = '#FFFFFF';
			}
			if (!stroke) {
				stroke = { css: 'rgba(0, 0, 0, 0.08)', weight: 1 };
			}
			// Subtle card drop shadow for artboards so they stand out from the canvas
			ctx.save();
			ctx.shadowColor = 'rgba(0, 0, 0, 0.06)';
			ctx.shadowBlur = 12 / view.zoom;
			ctx.shadowOffsetY = 4 / view.zoom;
			roundedRectPath(ctx, bounds.x, bounds.y, bounds.width, bounds.height, radius || 4);
			ctx.fillStyle = fillCss;
			ctx.fill();
			ctx.restore();
			filled = true;
		}

		if (node.type === 'TEXT') {
			const textColor = fillCss || '#1A1917';
			const weight = node.fontWeight || 400;
			const size = node.fontSize || 14;
			const family = node.fontFamily ? `${node.fontFamily}, sans-serif` : 'Inter, sans-serif';
			ctx.font = `${weight} ${size}px ${family}`;
			ctx.fillStyle = textColor;
			ctx.textBaseline = 'top';
			const text = node.text || (node as unknown as { content?: string }).content || '';
			const maxW = bounds.width > 0 ? bounds.width : 360;
			// Simple word wrap within bounds
			const words = text.split(/\s+/);
			let line = '';
			let lineY = bounds.y + 2;
			const lineHeight = node.lineHeight || Math.round(size * 1.38);
			const maxBottom = bounds.height > 0 ? bounds.y + bounds.height + lineHeight : Infinity;
			for (const word of words) {
				const testLine = line ? `${line} ${word}` : word;
				if (ctx.measureText(testLine).width > maxW && line) {
					ctx.fillText(line, bounds.x, lineY);
					line = word;
					lineY += lineHeight;
					if (lineY > maxBottom) break;
				} else {
					line = testLine;
				}
			}
			if (line && lineY <= maxBottom) {
				ctx.fillText(line, bounds.x, lineY);
			}
		} else {
			if (!filled && fillCss) {
				roundedRectPath(ctx, bounds.x, bounds.y, bounds.width, bounds.height, radius);
				ctx.fillStyle = fillCss;
				ctx.fill();
			}
			if (stroke) {
				roundedRectPath(ctx, bounds.x, bounds.y, bounds.width, bounds.height, radius);
				ctx.strokeStyle = stroke.css;
				ctx.lineWidth = stroke.weight;
				ctx.stroke();
			}
		}

		if (node.clipsContent && node.childIds.length > 0 && bounds.width > 0 && bounds.height > 0) {
			roundedRectPath(ctx, bounds.x, bounds.y, bounds.width, bounds.height, radius);
			ctx.clip();
		}

		// If this is an INSTANCE with no explicit child nodes, render master component children
		if (node.type === 'INSTANCE' && node.childIds.length === 0) {
			const compId = (node as unknown as { componentId?: string }).componentId;
			if (compId) {
				const comp = model.graph.getNode(compId);
				if (comp && comp.childIds.length > 0) {
					const compBounds = model.graph.getAbsoluteBounds(comp.id);
					ctx.save();
					ctx.translate(bounds.x - compBounds.x, bounds.y - compBounds.y);
					for (const childId of comp.childIds) {
						const child = model.graph.getNode(childId);
						if (child) drawSceneSubtree(child, false);
					}
					ctx.restore();
				}
			}
		}

		for (const childId of node.childIds) {
			const child = model.graph.getNode(childId);
			if (child) drawSceneSubtree(child, false);
		}
		ctx.restore();
	}

	for (const topNode of topNodes) {
		drawSceneSubtree(topNode, true);
	}

	// 2. Draw hover and selection outlines in OpenPencil / Figma style
	if (view.hoveredNodeId && view.hoveredNodeId !== view.selectedNodeId) {
		const hovered = model.graph.getNode(view.hoveredNodeId);
		if (hovered) {
			const b = model.graph.getAbsoluteBounds(hovered.id);
			const isComp = hovered.type === 'COMPONENT' || hovered.type === 'INSTANCE';
			ctx.save();
			ctx.strokeStyle = isComp ? componentCss : accentCss;
			ctx.lineWidth = 1.5 / view.zoom;
			ctx.strokeRect(b.x, b.y, b.width, b.height);
			ctx.restore();
		}
	}

	if (view.selectedNodeId) {
		const selected = model.graph.getNode(view.selectedNodeId);
		if (selected) {
			const b = model.graph.getAbsoluteBounds(selected.id);
			const isComp = selected.type === 'COMPONENT' || selected.type === 'INSTANCE';
			const outlineColor = isComp ? componentCss : accentCss;
			ctx.save();
			ctx.strokeStyle = outlineColor;
			ctx.lineWidth = 2 / view.zoom;
			ctx.strokeRect(b.x, b.y, b.width, b.height);
			// Corner handles
			const handleSize = 6 / view.zoom;
			ctx.fillStyle = '#FFFFFF';
			for (const [hx, hy] of [
				[b.x, b.y],
				[b.x + b.width, b.y],
				[b.x, b.y + b.height],
				[b.x + b.width, b.y + b.height]
			]) {
				ctx.fillRect(hx - handleSize / 2, hy - handleSize / 2, handleSize, handleSize);
				ctx.strokeRect(hx - handleSize / 2, hy - handleSize / 2, handleSize, handleSize);
			}
			ctx.restore();
		}
	}

	ctx.restore();
	ctx.restore();
}

export { parseColor };
