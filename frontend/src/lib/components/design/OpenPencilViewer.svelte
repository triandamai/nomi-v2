<script lang="ts">
	import { Copy, Download, Palette, ShieldAlert, X } from '@lucide/svelte';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';
	import { m } from '$lib/paraglide/messages';
	import { MAX_ZOOM, MIN_ZOOM, type CanvasTool } from '$lib/designStudio';
	import {
		exportStudioSceneToSVG,
		getStudioTargetBounds,
		hitTestStudioScene,
		paintOpenPencilSceneCanvas,
		type StudioHitResult,
		type StudioSceneModel
	} from '$lib/openPencilCanvas';

	/** A numbered feedback mark to draw over what it points at. */
	export type CanvasPin = { number: number; pagePath: string | null; elementId: string | null; x: number; y: number };

	let {
		model,
		tool = 'select',
		zoom = $bindable(1),
		panX = $bindable(0),
		panY = $bindable(0),
		onhit,
		onpickDirection,
		onpinclick,
		onemptyclick,
		selectedNodeId = null,
		pins = [],
		inspectorOpen = $bindable(false)
	}: {
		model: StudioSceneModel;
		tool?: CanvasTool;
		zoom?: number;
		panX?: number;
		panY?: number;
		/** A click on an element with the select or comment tool; `at` is where, inside the canvas. */
		onhit?: (hit: StudioHitResult, at: { x: number; y: number }) => void;
		onpickDirection?: (directionId: string) => void;
		onpinclick?: (pin: CanvasPin) => void;
		/** A click on the empty canvas: clears the selection, as in a design tool. */
		onemptyclick?: () => void;
		selectedNodeId?: string | null;
		pins?: CanvasPin[];
		inspectorOpen?: boolean;
	} = $props();

	// How far a press moves, in CSS pixels, before it is a drag rather than a click.
	const DRAG_THRESHOLD = 4;
	// How much one wheel notch with Ctrl/⌘ (or a pinch) zooms.
	const WHEEL_ZOOM_FACTOR = 1.1;
	// How far above a frame its name sits, in CSS pixels.
	const FRAME_LABEL_GAP = 6;

	let canvasEl = $state<HTMLCanvasElement | null>(null);
	let hostEl = $state<HTMLElement | null>(null);
	let hoveredNodeId = $state<string | null>(null);
	let inspectorTab = $state<'tokens' | 'lint' | 'layers'>('layers');
	let copiedSvg = $state(false);
	let spaceHeld = $state(false);
	let size = $state({ w: 0, h: 0 });

	let pointerDown = $state<{ x: number; y: number; originPanX: number; originPanY: number; moved: boolean } | null>(null);
	const panning = $derived(tool === 'move' || spaceHeld);
	const cursor = $derived(pointerDown?.moved ? 'grabbing' : panning ? 'grab' : tool === 'comment' ? 'crosshair' : 'default');

	function paint() {
		if (!canvasEl) return;
		paintOpenPencilSceneCanvas(canvasEl, model, { panX, panY, zoom, hoveredNodeId: panning ? null : hoveredNodeId, selectedNodeId });
	}

	$effect(() => {
		// Repaints whenever the view, the model or the canvas's size changes.
		void size.w;
		void size.h;
		paint();
	});

	$effect(() => {
		if (!hostEl) return;
		const observer = new ResizeObserver(([entry]) => {
			size = { w: entry.contentRect.width, h: entry.contentRect.height };
		});
		observer.observe(hostEl);
		return () => observer.disconnect();
	});

	function world(e: { clientX: number; clientY: number }) {
		const rect = canvasEl!.getBoundingClientRect();
		const x = e.clientX - rect.left;
		const y = e.clientY - rect.top;
		return { x, y, worldX: (x - panX) / zoom, worldY: (y - panY) / zoom };
	}

	function handlePointerDown(e: PointerEvent) {
		if (!canvasEl || (e.button !== 0 && e.button !== 1)) return;
		canvasEl.setPointerCapture(e.pointerId);
		pointerDown = { x: e.clientX, y: e.clientY, originPanX: panX, originPanY: panY, moved: e.button === 1 };
	}

	function handlePointerMove(e: PointerEvent) {
		if (!canvasEl) return;
		if (pointerDown) {
			const dx = e.clientX - pointerDown.x;
			const dy = e.clientY - pointerDown.y;
			if (!pointerDown.moved && Math.hypot(dx, dy) > DRAG_THRESHOLD) pointerDown.moved = true;
			if (pointerDown.moved) {
				panX = pointerDown.originPanX + dx;
				panY = pointerDown.originPanY + dy;
				return;
			}
		}
		if (panning) return;
		const at = world(e);
		hoveredNodeId = hitTestStudioScene(model, at.worldX, at.worldY)?.target.nodeId ?? null;
	}

	function handlePointerUp(e: PointerEvent) {
		if (!canvasEl) return;
		if (canvasEl.hasPointerCapture(e.pointerId)) canvasEl.releasePointerCapture(e.pointerId);
		const wasDrag = pointerDown?.moved ?? false;
		pointerDown = null;
		if (wasDrag || panning) return;

		const at = world(e);
		const hit = hitTestStudioScene(model, at.worldX, at.worldY);
		if (!hit) {
			onemptyclick?.();
			return;
		}
		if (hit.target.kind === 'direction-cta' && hit.target.directionId) {
			onpickDirection?.(hit.target.directionId);
			return;
		}
		onhit?.(hit, { x: at.x, y: at.y });
	}

	function handlePointerLeave() {
		hoveredNodeId = null;
	}

	function handleWheel(e: WheelEvent) {
		e.preventDefault();
		if (e.ctrlKey || e.metaKey) {
			// Zooms around the pointer, so what is under it stays under it.
			const at = world(e);
			const next = Math.round(Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, zoom * (e.deltaY < 0 ? WHEEL_ZOOM_FACTOR : 1 / WHEEL_ZOOM_FACTOR))) * 100) / 100;
			panX = at.x - at.worldX * next;
			panY = at.y - at.worldY * next;
			zoom = next;
		} else {
			panX -= e.shiftKey ? e.deltaY : e.deltaX;
			panY -= e.shiftKey ? 0 : e.deltaY;
		}
	}

	function typing(target: EventTarget | null) {
		const el = target as HTMLElement | null;
		const tag = el?.tagName?.toLowerCase();
		return tag === 'input' || tag === 'textarea' || tag === 'select' || !!el?.isContentEditable;
	}

	function keydown(e: KeyboardEvent) {
		if (e.code === 'Space' && !typing(e.target)) {
			if (!spaceHeld) spaceHeld = true;
			e.preventDefault();
		}
	}

	function keyup(e: KeyboardEvent) {
		if (e.code === 'Space') spaceHeld = false;
	}

	// Frame names sit above each frame, as in a design tool, and move with the canvas.
	const frameLabels = $derived(
		model.frames.map((frame) => ({
			id: frame.id,
			title: frame.title,
			left: panX + frame.x * zoom,
			top: panY + frame.y * zoom - FRAME_LABEL_GAP,
			width: frame.width * zoom
		}))
	);

	// Feedback marks, placed at the point they were dropped on, inside what they point at.
	const placedPins = $derived(
		pins.flatMap((pin) => {
			const bounds = getStudioTargetBounds(model, pin.pagePath, pin.elementId);
			if (!bounds) return [];
			return [{ pin, left: panX + (bounds.x + bounds.width * pin.x) * zoom, top: panY + (bounds.y + bounds.height * pin.y) * zoom }];
		})
	);

	async function copySceneSvg() {
		const svg = exportStudioSceneToSVG(model);
		if (!svg) return;
		try {
			await navigator.clipboard.writeText(svg);
			copiedSvg = true;
			setTimeout(() => (copiedSvg = false), 2000);
		} catch {
			// The clipboard can be refused; downloading still works.
		}
	}

	function downloadSceneSvg() {
		const svg = exportStudioSceneToSVG(model);
		if (!svg) return;
		const url = URL.createObjectURL(new Blob([svg], { type: 'image/svg+xml' }));
		const a = document.createElement('a');
		a.href = url;
		a.download = `design-${model.pageId || 'canvas'}.svg`;
		a.click();
		URL.revokeObjectURL(url);
	}
</script>

<svelte:window onkeydown={keydown} onkeyup={keyup} onblur={() => (spaceHeld = false)} />

<div bind:this={hostEl} class="viewer">
	<canvas
		bind:this={canvasEl}
		class="viewer__canvas"
		style:cursor
		onpointerdown={handlePointerDown}
		onpointermove={handlePointerMove}
		onpointerup={handlePointerUp}
		onpointerleave={handlePointerLeave}
		onwheel={handleWheel}
		data-testid="openpencil-canvas"
		data-tool={tool}
		aria-label={m.studio_canvas_openpencil()}
	></canvas>

	<div class="viewer__overlay" aria-hidden="true">
		{#each frameLabels as label (label.id)}
			<span class="viewer__frame-label" style:left={`${label.left}px`} style:top={`${label.top}px`} style:max-width={`${Math.max(40, label.width)}px`}>{label.title}</span>
		{/each}
	</div>

	{#each placedPins as placed (placed.pin.number)}
		<button
			type="button"
			class="viewer__pin"
			style:left={`${placed.left}px`}
			style:top={`${placed.top}px`}
			aria-label={m.studio_pin_label({ number: placed.pin.number })}
			onclick={() => onpinclick?.(placed.pin)}
		>
			{placed.pin.number}
		</button>
	{/each}

	{#if inspectorOpen}
		<aside class="viewer__inspector" aria-label={m.studio_inspector()} data-testid="openpencil-inspector">
			<div class="viewer__inspector-head">
				<strong class="md-title-small">{m.studio_inspector()}</strong>
				<div class="viewer__inspector-actions">
					<button type="button" class="viewer__icon" aria-label={copiedSvg ? m.copy_done() : m.studio_openpencil_copy_svg()} title={copiedSvg ? m.copy_done() : m.studio_openpencil_copy_svg()} onclick={copySceneSvg}><Icon icon={Copy} size={16} /></button>
					<button type="button" class="viewer__icon" aria-label={m.studio_openpencil_export_svg()} title={m.studio_openpencil_export_svg()} onclick={downloadSceneSvg}><Icon icon={Download} size={16} /></button>
					<button type="button" class="viewer__icon" aria-label={m.studio_inspector_close()} title={m.studio_inspector_close()} onclick={() => (inspectorOpen = false)}><Icon icon={X} size={16} /></button>
				</div>
			</div>
			<div class="viewer__tabs" role="tablist">
				<button type="button" role="tab" aria-selected={inspectorTab === 'layers'} onclick={() => (inspectorTab = 'layers')}>{m.studio_inspector_frames({ count: model.frames.length })}</button>
				<button type="button" role="tab" aria-selected={inspectorTab === 'tokens'} onclick={() => (inspectorTab = 'tokens')}>{m.studio_openpencil_tokens()} ({model.variables.length})</button>
				<button type="button" role="tab" aria-selected={inspectorTab === 'lint'} onclick={() => (inspectorTab = 'lint')}>{m.studio_inspector_checks({ count: model.lintIssues.length })}</button>
			</div>
			<div class="viewer__inspector-body">
				{#if inspectorTab === 'tokens'}
					{#if model.variables.length === 0}
						<p class="md-body-small viewer__muted">{m.studio_inspector_no_tokens()}</p>
					{:else}
						<ul class="viewer__rows">
							{#each model.variables as v (v.id)}
								<li class="viewer__row">
									{#if v.type === 'COLOR'}
										<span class="viewer__chip" style:background-color={v.cssValue}></span>
									{:else}
										<span class="viewer__muted"><Icon icon={Palette} size={14} /></span>
									{/if}
									<span class="viewer__row-name">{v.name}</span>
									<code class="viewer__mono">{v.cssValue}</code>
								</li>
							{/each}
						</ul>
					{/if}
				{:else if inspectorTab === 'layers'}
					<ul class="viewer__rows">
						{#each model.frames as frame (frame.id)}
							<li class="viewer__row">
								<span class="viewer__row-name">{frame.title}</span>
								<code class="viewer__mono">{Math.round(frame.width)} × {Math.round(frame.height)}</code>
							</li>
						{/each}
					</ul>
				{:else if model.lintIssues.length === 0}
					<p class="md-body-small viewer__muted">{m.studio_inspector_no_issues()}</p>
				{:else}
					<ul class="viewer__rows">
						{#each model.lintIssues as issue (`${issue.nodeId}-${issue.ruleId}`)}
							<li class="viewer__row viewer__row--issue">
								<span class="viewer__muted"><Icon icon={ShieldAlert} size={14} /></span>
								<span class="md-body-small">{issue.message}</span>
							</li>
						{/each}
					</ul>
				{/if}
			</div>
		</aside>
	{/if}
</div>

<style>
	.viewer {
		position: relative;
		flex: 1;
		min-height: 0;
		width: 100%;
		height: 100%;
		overflow: hidden;
		background-color: var(--md-sys-color-surface-container);
	}
	.viewer__canvas {
		display: block;
		width: 100%;
		height: 100%;
		touch-action: none;
		user-select: none;
	}
	.viewer__overlay {
		position: absolute;
		inset: 0;
		pointer-events: none;
		overflow: hidden;
	}
	.viewer__frame-label {
		position: absolute;
		transform: translateY(-100%);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: 12px;
		line-height: 16px;
		color: var(--md-sys-color-on-surface-variant);
	}
	.viewer__pin {
		position: absolute;
		z-index: 5;
		display: grid;
		place-items: center;
		min-width: 28px;
		height: 28px;
		padding: 0 6px;
		transform: translate(-4px, -100%);
		border: 2px solid var(--md-sys-color-surface);
		border-radius: 14px 14px 14px 2px;
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
		font: 600 12px/1 inherit;
		cursor: pointer;
		box-shadow: 0 2px 8px color-mix(in srgb, var(--md-sys-color-shadow) 24%, transparent);
	}
	.viewer__pin:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.viewer__inspector {
		position: absolute;
		top: 12px;
		right: 12px;
		bottom: 72px;
		z-index: 20;
		display: flex;
		flex-direction: column;
		width: min(300px, calc(100% - 24px));
		overflow: hidden;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-lowest);
		box-shadow: 0 8px 24px color-mix(in srgb, var(--md-sys-color-shadow) 16%, transparent);
	}
	.viewer__inspector-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 8px 8px 16px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.viewer__inspector-actions {
		display: flex;
		gap: 2px;
	}
	.viewer__icon {
		display: grid;
		place-items: center;
		width: 32px;
		height: 32px;
		border: 0;
		border-radius: var(--md-sys-shape-corner-small);
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		cursor: pointer;
	}
	.viewer__icon:hover,
	.viewer__tabs button:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.viewer__icon:focus-visible,
	.viewer__tabs button:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.viewer__tabs {
		display: flex;
		gap: 2px;
		padding: 4px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.viewer__tabs button {
		flex: 1;
		height: 32px;
		border: 0;
		border-radius: var(--md-sys-shape-corner-small);
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-size: 12px;
		cursor: pointer;
	}
	.viewer__tabs button[aria-selected='true'] {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.viewer__inspector-body {
		flex: 1;
		overflow-y: auto;
		padding: 8px;
	}
	.viewer__rows {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.viewer__row {
		display: flex;
		align-items: center;
		gap: 8px;
		min-height: 36px;
		padding: 4px 8px;
		border-radius: var(--md-sys-shape-corner-small);
	}
	.viewer__row--issue {
		align-items: flex-start;
	}
	.viewer__row-name {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: 13px;
	}
	.viewer__chip {
		width: 14px;
		height: 14px;
		flex: none;
		border-radius: 4px;
		border: 1px solid var(--md-sys-color-outline-variant);
	}
	.viewer__mono {
		font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
		font-size: 11px;
		color: var(--md-sys-color-on-surface-variant);
	}
	.viewer__muted {
		margin: 0;
		padding: 8px;
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
