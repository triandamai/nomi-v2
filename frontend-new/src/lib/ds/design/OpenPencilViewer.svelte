<script lang="ts">
	import { Copy, Download, Layers, Palette, ShieldAlert } from '@lucide/svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import { m } from '$lib/paraglide/messages';
	import {
		exportStudioSceneToSVG,
		hitTestStudioScene,
		paintOpenPencilSceneCanvas,
		type StudioHitResult,
		type StudioSceneModel
	} from '$lib/design/openPencilCanvas';

	let {
		model,
		zoom = $bindable(1),
		panX = $bindable(0),
		panY = $bindable(0),
		onhit,
		onpickDirection,
		selectedNodeId = null
	}: {
		model: StudioSceneModel;
		zoom?: number;
		panX?: number;
		panY?: number;
		onhit?: (hit: StudioHitResult) => void;
		onpickDirection?: (directionId: string) => void;
		selectedNodeId?: string | null;
	} = $props();

	let canvasEl = $state<HTMLCanvasElement | null>(null);
	let hoveredNodeId = $state<string | null>(null);
	let inspectorTab = $state<'tokens' | 'lint' | 'layers'>('tokens');
	let inspectorOpen = $state(true);
	let copiedSvg = $state(false);

	let pointerDown = $state<{ x: number; y: number; originPanX: number; originPanY: number; moved: boolean } | null>(null);

	$effect(() => {
		if (!canvasEl) return;
		paintOpenPencilSceneCanvas(canvasEl, model, {
			panX,
			panY,
			zoom,
			hoveredNodeId,
			selectedNodeId
		});
	});

	function handlePointerDown(e: PointerEvent) {
		if (!canvasEl || e.button !== 0) return;
		canvasEl.setPointerCapture(e.pointerId);
		pointerDown = {
			x: e.clientX,
			y: e.clientY,
			originPanX: panX,
			originPanY: panY,
			moved: false
		};
	}

	function handlePointerMove(e: PointerEvent) {
		if (!canvasEl) return;
		if (pointerDown) {
			const dx = e.clientX - pointerDown.x;
			const dy = e.clientY - pointerDown.y;
			if (!pointerDown.moved && Math.hypot(dx, dy) > 4) {
				pointerDown.moved = true;
			}
			if (pointerDown.moved) {
				panX = pointerDown.originPanX + dx;
				panY = pointerDown.originPanY + dy;
				return;
			}
		}

		const rect = canvasEl.getBoundingClientRect();
		const clientX = e.clientX - rect.left;
		const clientY = e.clientY - rect.top;
		const worldX = (clientX - panX) / zoom;
		const worldY = (clientY - panY) / zoom;
		const hit = hitTestStudioScene(model, worldX, worldY);
		hoveredNodeId = hit?.target.nodeId ?? null;
		if (canvasEl) {
			paintOpenPencilSceneCanvas(canvasEl, model, {
				panX,
				panY,
				zoom,
				hoveredNodeId,
				selectedNodeId
			});
		}
	}

	function handlePointerUp(e: PointerEvent) {
		if (!canvasEl) return;
		if (canvasEl.hasPointerCapture(e.pointerId)) {
			canvasEl.releasePointerCapture(e.pointerId);
		}
		const wasDrag = pointerDown?.moved ?? false;
		pointerDown = null;

		if (wasDrag) return;

		const rect = canvasEl.getBoundingClientRect();
		const clientX = e.clientX - rect.left;
		const clientY = e.clientY - rect.top;
		const worldX = (clientX - panX) / zoom;
		const worldY = (clientY - panY) / zoom;
		const hit = hitTestStudioScene(model, worldX, worldY);
		if (!hit) return;

		if (hit.target.kind === 'direction-cta' && hit.target.directionId) {
			onpickDirection?.(hit.target.directionId);
			return;
		}

		onhit?.(hit);
	}

	function handlePointerLeave() {
		hoveredNodeId = null;
		pointerDown = null;
		if (canvasEl) {
			paintOpenPencilSceneCanvas(canvasEl, model, {
				panX,
				panY,
				zoom,
				hoveredNodeId: null,
				selectedNodeId
			});
		}
	}

	function handleWheel(e: WheelEvent) {
		if (e.ctrlKey || e.metaKey) {
			e.preventDefault();
			const zoomFactor = e.deltaY < 0 ? 1.1 : 0.9;
			zoom = Math.round(Math.min(2, Math.max(0.25, zoom * zoomFactor)) * 100) / 100;
		} else {
			panX -= e.deltaX;
			panY -= e.deltaY;
		}
	}

	async function copySceneSvg() {
		const svg = exportStudioSceneToSVG(model);
		if (!svg) return;
		try {
			await navigator.clipboard.writeText(svg);
			copiedSvg = true;
			setTimeout(() => {
				copiedSvg = false;
			}, 2000);
		} catch {
			// fallback
		}
	}

	function downloadSceneSvg() {
		const svg = exportStudioSceneToSVG(model);
		if (!svg) return;
		const blob = new Blob([svg], { type: 'image/svg+xml' });
		const url = URL.createObjectURL(blob);
		const a = document.createElement('a');
		a.href = url;
		a.download = `openpencil-${model.pageId || 'design'}.svg`;
		a.click();
		URL.revokeObjectURL(url);
	}
</script>

<div class="relative flex size-full min-h-0 flex-1 overflow-hidden bg-bg">
	<!-- OpenPencil Native 60fps Vector Canvas -->
	<canvas
		bind:this={canvasEl}
		class="size-full cursor-crosshair touch-none select-none"
		onpointerdown={handlePointerDown}
		onpointermove={handlePointerMove}
		onpointerup={handlePointerUp}
		onpointerleave={handlePointerLeave}
		onwheel={handleWheel}
		data-testid="openpencil-canvas"
		aria-label={m.studio_canvas_openpencil()}
	></canvas>

	<!-- OpenPencil Rulers / Scene-Graph Inspector Sheet -->
	<aside
		class={[
			'absolute top-3 right-3 bottom-3 z-20 flex w-76 flex-col overflow-hidden rounded-md bg-surface shadow-float outline -outline-offset-1 outline-border transition-transform',
			inspectorOpen ? 'translate-x-0' : 'translate-x-80 pointer-events-none opacity-0'
		]}
		aria-label={m.studio_openpencil_tokens()}
		data-testid="openpencil-inspector"
	>
		<div class="flex items-center justify-between border-b border-border p-3">
			<div class="flex items-center gap-1.5 font-mono text-label text-text">
				<span class="text-accent"><Icon icon={Layers} size={16} /></span>
				<span class="font-semibold">OpenPencil Scene Graph</span>
			</div>
			<div class="flex items-center gap-1">
				<button
					type="button"
					class="cursor-pointer rounded-xs p-1 text-text-2 hover:bg-surface-2 hover:text-text"
					title={copiedSvg ? m.copy_done() : m.studio_openpencil_export_svg()}
					onclick={copySceneSvg}
				>
					<Icon icon={Copy} size={14} />
				</button>
				<button
					type="button"
					class="cursor-pointer rounded-xs p-1 text-text-2 hover:bg-surface-2 hover:text-text"
					title={m.studio_openpencil_export_svg()}
					onclick={downloadSceneSvg}
				>
					<Icon icon={Download} size={14} />
				</button>
			</div>
		</div>

		<!-- Inspector Tabs -->
		<div class="flex border-b border-border bg-surface-2 p-1 font-mono text-micro">
			<button
				type="button"
				class={[
					'flex-1 cursor-pointer rounded-xs py-1 text-center transition-colors',
					inspectorTab === 'tokens'
						? 'bg-surface font-medium text-text outline -outline-offset-1 outline-border-strong'
						: 'text-text-3 hover:text-text'
				]}
				onclick={() => (inspectorTab = 'tokens')}
			>
				{m.studio_openpencil_tokens()} ({model.variables.length})
			</button>
			<button
				type="button"
				class={[
					'flex-1 cursor-pointer rounded-xs py-1 text-center transition-colors',
					inspectorTab === 'layers'
						? 'bg-surface font-medium text-text outline -outline-offset-1 outline-border-strong'
						: 'text-text-3 hover:text-text'
				]}
				onclick={() => (inspectorTab = 'layers')}
			>
				Frames ({model.frames.length})
			</button>
			<button
				type="button"
				class={[
					'flex-1 cursor-pointer rounded-xs py-1 text-center transition-colors',
					inspectorTab === 'lint'
						? 'bg-surface font-medium text-text outline -outline-offset-1 outline-border-strong'
						: 'text-text-3 hover:text-text'
				]}
				onclick={() => (inspectorTab = 'lint')}
			>
				Lint ({model.lintIssues.length})
			</button>
		</div>

		<!-- Tab Content -->
		<div class="flex flex-1 flex-col gap-2 overflow-y-auto p-3">
			{#if inspectorTab === 'tokens'}
				<div class="flex flex-col gap-2">
					<span class="font-mono text-micro text-text-3 uppercase">Design Variables</span>
					{#if model.variables.length === 0}
						<p class="m-0 text-caption text-text-2">No variables bound.</p>
					{:else}
						<ul class="m-0 flex list-none flex-col gap-1.5 p-0">
							{#each model.variables as v (v.id)}
								<li
									class="flex items-center justify-between gap-2 rounded-xs bg-surface-2 px-2 py-1.5 font-mono text-caption outline -outline-offset-1 outline-border"
								>
									<div class="flex items-center gap-1.5 truncate">
										{#if v.type === 'COLOR'}
											<span
												class="size-3 shrink-0 rounded-xs outline -outline-offset-1 outline-border"
												style:background-color={v.cssValue}
											></span>
										{:else}
											<span class="text-text-3"><Icon icon={Palette} size={12} /></span>
										{/if}
										<span class="truncate text-text font-medium">{v.name}</span>
									</div>
									<span class="shrink-0 text-micro text-text-3">{v.cssValue}</span>
								</li>
							{/each}
						</ul>
					{/if}
				</div>
			{:else if inspectorTab === 'layers'}
				<div class="flex flex-col gap-2">
					<span class="font-mono text-micro text-text-3 uppercase">Top-Level Frames</span>
					<ul class="m-0 flex list-none flex-col gap-1.5 p-0">
						{#each model.frames as frame (frame.id)}
							<li
								class="flex flex-col gap-1 rounded-xs bg-surface-2 p-2 outline -outline-offset-1 outline-border"
							>
								<div class="flex items-center justify-between">
									<strong class="text-caption font-semibold text-text">{frame.title}</strong>
									<span class="font-mono text-micro text-accent uppercase">{frame.kind}</span>
								</div>
								<span class="font-mono text-micro text-text-3"
									>{Math.round(frame.width)} × {Math.round(frame.height)} px</span
								>
							</li>
						{/each}
					</ul>
				</div>
			{:else if inspectorTab === 'lint'}
				<div class="flex flex-col gap-2">
					<span class="font-mono text-micro text-text-3 uppercase">OpenPencil Linter</span>
					{#if model.lintIssues.length === 0}
						<p class="m-0 text-caption text-text-2">No design lint issues detected! 🎉</p>
					{:else}
						<ul class="m-0 flex list-none flex-col gap-1.5 p-0">
							{#each model.lintIssues as issue (`${issue.nodeId}-${issue.ruleId}`)}
								<li
									class="flex flex-col gap-1 rounded-xs bg-surface-2 p-2 outline -outline-offset-1 outline-border"
								>
									<div class="flex items-center gap-1.5">
										<span class="text-accent"><Icon icon={ShieldAlert} size={14} /></span>
										<span class="font-mono text-micro font-medium text-text">{issue.ruleId}</span>
									</div>
									<p class="m-0 text-micro text-text-2">{issue.message}</p>
								</li>
							{/each}
						</ul>
					{/if}
				</div>
			{/if}
		</div>
	</aside>
</div>
