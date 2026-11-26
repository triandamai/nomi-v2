<script lang="ts">
	import { Copy, Download, Palette, ShieldAlert, X } from '@lucide/svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import { m } from '$lib/paraglide/messages';
	import { MAX_ZOOM, MIN_ZOOM, type CanvasTool } from '$lib/design/designStudio';
	import {
		exportStudioSceneToSVG,
		getStudioTargetBounds,
		hitTestStudioScene,
		paintOpenPencilSceneCanvas,
		type StudioHitResult,
		type StudioSceneModel
	} from '$lib/design/openPencilCanvas';

	import type { SoraCanvasActivity } from '$lib/design/soraActivity';

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
		inspectorOpen = $bindable(false),
		soraActivity = null
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
		/** Live activity from Sora so her multiplayer cursor glides over what she is designing. */
		soraActivity?: SoraCanvasActivity | null;
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
	let soraDrift = $state(0);

	// Track active pointers for single-finger drag panning AND two-finger pinch-to-zoom on touch screens.
	const activePointers = new Map<number, { x: number; y: number }>();
	let pinchState: { dist: number; midX: number; midY: number; zoom: number; panX: number; panY: number } | null = null;
	let pointerDown = $state<{ pointerId: number; x: number; y: number; originPanX: number; originPanY: number; moved: boolean } | null>(null);
	const panning = $derived(tool === 'move' || spaceHeld);
	const cursor = $derived(pointerDown?.moved ? 'grabbing' : panning ? 'grab' : tool === 'comment' ? 'crosshair' : 'default');

	// Gentle organic drift tick while Sora is actively working on elements not yet in the scene graph.
	$effect(() => {
		if (!soraActivity?.active) return;
		const timer = setInterval(() => {
			soraDrift += 1;
		}, 1400);
		return () => clearInterval(timer);
	});

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

	function startPinch() {
		const pts = [...activePointers.values()];
		if (pts.length < 2) return;
		const [a, b] = pts;
		pinchState = {
			dist: Math.max(1, Math.hypot(b.x - a.x, b.y - a.y)),
			midX: (a.x + b.x) / 2,
			midY: (a.y + b.y) / 2,
			zoom,
			panX,
			panY,
		};
		pointerDown = null;
	}

	function handlePointerDown(e: PointerEvent) {
		if (!canvasEl) return;
		if (e.pointerType === 'mouse' && e.button !== 0 && e.button !== 1) return;
		canvasEl.setPointerCapture(e.pointerId);
		activePointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
		if (activePointers.size >= 2) {
			startPinch();
			return;
		}
		pointerDown = { pointerId: e.pointerId, x: e.clientX, y: e.clientY, originPanX: panX, originPanY: panY, moved: e.button === 1 };
	}

	function handlePointerMove(e: PointerEvent) {
		if (!canvasEl) return;
		if (activePointers.has(e.pointerId)) {
			activePointers.set(e.pointerId, { x: e.clientX, y: e.clientY });
		}
		if (pinchState && activePointers.size >= 2) {
			const [a, b] = [...activePointers.values()];
			const dist = Math.max(1, Math.hypot(b.x - a.x, b.y - a.y));
			const midX = (a.x + b.x) / 2;
			const midY = (a.y + b.y) / 2;
			const rect = canvasEl.getBoundingClientRect();
			const startLocalX = pinchState.midX - rect.left;
			const startLocalY = pinchState.midY - rect.top;
			const localX = midX - rect.left;
			const localY = midY - rect.top;
			const worldX = (startLocalX - pinchState.panX) / pinchState.zoom;
			const worldY = (startLocalY - pinchState.panY) / pinchState.zoom;
			const nextZoom = Math.round(Math.min(MAX_ZOOM, Math.max(MIN_ZOOM, pinchState.zoom * (dist / pinchState.dist))) * 100) / 100;
			panX = localX - worldX * nextZoom;
			panY = localY - worldY * nextZoom;
			zoom = nextZoom;
			return;
		}
		if (pointerDown && pointerDown.pointerId === e.pointerId) {
			const dx = e.clientX - pointerDown.x;
			const dy = e.clientY - pointerDown.y;
			if (!pointerDown.moved && Math.hypot(dx, dy) > DRAG_THRESHOLD) pointerDown.moved = true;
			if (pointerDown.moved) {
				panX = pointerDown.originPanX + dx;
				panY = pointerDown.originPanY + dy;
				return;
			}
		}
		if (panning || e.pointerType === 'touch') return;
		const at = world(e);
		hoveredNodeId = hitTestStudioScene(model, at.worldX, at.worldY)?.target.nodeId ?? null;
	}

	function handlePointerUp(e: PointerEvent) {
		if (!canvasEl) return;
		if (canvasEl.hasPointerCapture(e.pointerId)) canvasEl.releasePointerCapture(e.pointerId);
		activePointers.delete(e.pointerId);
		if (pinchState) {
			if (activePointers.size < 2) {
				pinchState = null;
				// Smoothly hand off the remaining finger to single-finger pan.
				const remaining = activePointers.entries().next().value as [number, { x: number; y: number }] | undefined;
				if (remaining) {
					pointerDown = { pointerId: remaining[0], x: remaining[1].x, y: remaining[1].y, originPanX: panX, originPanY: panY, moved: true };
				}
			}
			return;
		}
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

	function handlePointerCancel(e: PointerEvent) {
		if (canvasEl?.hasPointerCapture(e.pointerId)) canvasEl.releasePointerCapture(e.pointerId);
		activePointers.delete(e.pointerId);
		if (activePointers.size < 2) pinchState = null;
		if (pointerDown?.pointerId === e.pointerId) pointerDown = null;
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

	// Sora's live multiplayer cursor: snaps to the element/screen she is creating or inspecting,
	// or glides smoothly over the active artboard while she drafts new elements.
	const soraCursor = $derived.by(() => {
		if (!soraActivity?.active) return null;
		const bounds = soraActivity.targetId ? getStudioTargetBounds(model, null, soraActivity.targetId) : null;
		if (bounds) {
			const box = {
				left: panX + bounds.x * zoom,
				top: panY + bounds.y * zoom,
				width: Math.max(24, bounds.width * zoom),
				height: Math.max(24, bounds.height * zoom),
			};
			return {
				x: box.left + box.width * 0.62,
				y: box.top + box.height * 0.42,
				label: soraActivity.label || 'Designing…',
				box,
			};
		}
		const step = soraActivity.tick + soraDrift;
		const frame = model.frames[step % Math.max(1, model.frames.length)];
		const phase = step * 1.15;
		if (frame) {
			const fx = frame.x + frame.width * (0.28 + 0.44 * (0.5 + 0.5 * Math.sin(phase)));
			const fy = frame.y + frame.height * (0.22 + 0.48 * (0.5 + 0.5 * Math.cos(phase * 0.85)));
			return {
				x: panX + fx * zoom,
				y: panY + fy * zoom,
				label: soraActivity.label || 'Designing…',
				box: null,
			};
		}
		const vw = size.w || 640;
		const vh = size.h || 420;
		return {
			x: vw * (0.35 + 0.3 * (0.5 + 0.5 * Math.sin(phase))),
			y: vh * (0.32 + 0.28 * (0.5 + 0.5 * Math.cos(phase * 0.8))),
			label: soraActivity.label || 'Designing…',
			box: null,
		};
	});

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
		onpointercancel={handlePointerCancel}
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
		{#if soraCursor?.box}
			<div
				class="viewer__sora-box"
				style:left={`${soraCursor.box.left}px`}
				style:top={`${soraCursor.box.top}px`}
				style:width={`${soraCursor.box.width}px`}
				style:height={`${soraCursor.box.height}px`}
			></div>
		{/if}
		{#if soraCursor}
			<div
				class="viewer__sora-cursor"
				data-testid="sora-live-cursor"
				style:transform={`translate3d(${soraCursor.x}px, ${soraCursor.y}px, 0)`}
			>
				<svg class="viewer__sora-pointer" width="18" height="20" viewBox="0 0 18 20" fill="none">
					<path d="M1.5 1.5L16.2 9.4L9.7 11.4L6.8 18.2L1.5 1.5Z" fill="var(--color-accent)" stroke="var(--color-surface)" stroke-width="1.5" stroke-linejoin="round" />
				</svg>
				<span class="viewer__sora-pill">
					<span class="viewer__sora-dot"></span>
					<strong class="viewer__sora-name">Sora</strong>
					<span class="viewer__sora-label">{soraCursor.label}</span>
				</span>
			</div>
		{/if}
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
				<strong class="text-ui font-semibold">{m.studio_inspector()}</strong>
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
						<p class="viewer__muted text-caption">{m.studio_inspector_no_tokens()}</p>
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
					<p class="viewer__muted text-caption">{m.studio_inspector_no_issues()}</p>
				{:else}
					<ul class="viewer__rows">
						{#each model.lintIssues as issue (`${issue.nodeId}-${issue.ruleId}`)}
							<li class="viewer__row viewer__row--issue">
								<span class="viewer__muted"><Icon icon={ShieldAlert} size={14} /></span>
								<span class="text-caption">{issue.message}</span>
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
		background-color: var(--color-bg);
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
		color: var(--color-text-2);
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
		border: 2px solid var(--color-surface);
		border-radius: 14px 14px 14px 2px;
		background: var(--color-accent);
		color: var(--color-accent-ink);
		font: 600 12px/1 inherit;
		cursor: pointer;
		box-shadow: var(--shadow-float);
	}
	.viewer__pin:focus-visible {
		outline: 2px solid var(--color-accent);
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
		border: 1px solid var(--color-border);
		border-radius: var(--radius-md);
		background: var(--color-surface);
		box-shadow: var(--shadow-float);
	}
	.viewer__inspector-head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 8px 8px 8px 16px;
		border-bottom: 1px solid var(--color-border);
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
		border-radius: var(--radius-sm);
		background: none;
		color: var(--color-text-2);
		cursor: pointer;
	}
	.viewer__icon:hover,
	.viewer__tabs button:hover {
		background: var(--color-surface-2);
	}
	.viewer__icon:focus-visible,
	.viewer__tabs button:focus-visible {
		outline: 2px solid var(--color-accent);
		outline-offset: -2px;
	}
	.viewer__tabs {
		display: flex;
		gap: 2px;
		padding: 4px;
		border-bottom: 1px solid var(--color-border);
	}
	.viewer__tabs button {
		flex: 1;
		height: 32px;
		border: 0;
		border-radius: var(--radius-sm);
		background: none;
		color: var(--color-text-2);
		font: inherit;
		font-size: 12px;
		cursor: pointer;
	}
	.viewer__tabs button[aria-selected='true'] {
		background: var(--color-surface-2);
		color: var(--color-text);
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
		border-radius: var(--radius-sm);
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
		border: 1px solid var(--color-border);
	}
	.viewer__mono {
		font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
		font-size: 11px;
		color: var(--color-text-2);
	}
	.viewer__muted {
		margin: 0;
		padding: 8px;
		color: var(--color-text-2);
	}
	.viewer__sora-box {
		position: absolute;
		border: 2px solid var(--color-accent);
		border-radius: 6px;
		background: color-mix(in srgb, var(--color-accent) 10%, transparent);
		transition: left 360ms cubic-bezier(0.22, 1, 0.36, 1), top 360ms cubic-bezier(0.22, 1, 0.36, 1), width 360ms cubic-bezier(0.22, 1, 0.36, 1), height 360ms cubic-bezier(0.22, 1, 0.36, 1);
		animation: sora-pulse 1.6s ease-in-out infinite;
	}
	.viewer__sora-cursor {
		position: absolute;
		top: 0;
		left: 0;
		z-index: 15;
		display: inline-flex;
		align-items: flex-start;
		gap: 4px;
		will-change: transform;
		transition: transform 420ms cubic-bezier(0.22, 1, 0.36, 1);
	}
	.viewer__sora-pointer {
		flex: none;
		filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.25));
	}
	.viewer__sora-pill {
		margin-top: 12px;
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 4px 10px 4px 8px;
		border-radius: 999px;
		background: var(--color-accent);
		color: var(--color-accent-ink);
		font-size: 12px;
		line-height: 16px;
		white-space: nowrap;
		box-shadow: var(--shadow-float);
	}
	.viewer__sora-dot {
		width: 6px;
		height: 6px;
		border-radius: 999px;
		background: currentColor;
		animation: sora-dot 1.1s ease-in-out infinite;
	}
	.viewer__sora-name {
		font-weight: 600;
	}
	.viewer__sora-label {
		opacity: 0.92;
		max-width: 180px;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	@keyframes sora-pulse {
		0%, 100% { opacity: 0.95; }
		50% { opacity: 0.45; }
	}
	@keyframes sora-dot {
		0%, 100% { transform: scale(1); opacity: 1; }
		50% { transform: scale(0.55); opacity: 0.45; }
	}
</style>
