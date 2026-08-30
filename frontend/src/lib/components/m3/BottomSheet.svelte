<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		open = $bindable(false),
		children,
		class: extraClass = '',
	}: {
		open?: boolean;
		children: Snippet;
		class?: string;
	} = $props();

	// A drag past this fraction of the sheet's own height counts as "dismiss"; anything
	// less snaps back. 0.25 matches MD3's own bottom-sheet dismissal threshold.
	const DISMISS_FRACTION = 0.25;

	let dialogEl: HTMLDialogElement | undefined = $state();
	let panelEl: HTMLDivElement | undefined = $state();
	let dragging = $state(false);

	let startY = 0;
	let panelHeight = 0;
	let activePointerId: number | undefined;

	$effect(() => {
		if (!dialogEl) return;
		if (open && !dialogEl.open) {
			dialogEl.showModal();
		} else if (!open && dialogEl.open) {
			dialogEl.close();
		}
	});

	function handleClose() {
		open = false;
	}

	function handleDialogClick(event: MouseEvent) {
		if (event.target === dialogEl) open = false;
	}

	function handlePointerDown(event: PointerEvent) {
		if (!panelEl) return;
		dragging = true;
		startY = event.clientY;
		panelHeight = panelEl.getBoundingClientRect().height;
		activePointerId = event.pointerId;
		(event.currentTarget as HTMLElement).setPointerCapture(event.pointerId);
	}

	function handlePointerMove(event: PointerEvent) {
		if (!dragging || !panelEl || event.pointerId !== activePointerId) return;
		const deltaY = Math.max(0, event.clientY - startY);
		panelEl.style.transform = `translateY(${deltaY}px)`;
	}

	function endDrag(event: PointerEvent) {
		if (!dragging || !panelEl || event.pointerId !== activePointerId) return;
		dragging = false;
		activePointerId = undefined;
		const deltaY = Math.max(0, event.clientY - startY);
		if (panelHeight > 0 && deltaY / panelHeight > DISMISS_FRACTION) {
			// Let the snap-closed transition play, then tell the dialog to actually close —
			// closing immediately would cut the animation off mid-flight.
			panelEl.style.transform = `translateY(100%)`;
			setTimeout(() => {
				open = false;
				if (panelEl) panelEl.style.transform = '';
			}, 200);
		} else {
			panelEl.style.transform = '';
		}
	}
</script>

<dialog
	bind:this={dialogEl}
	class="m3-bottom-sheet {extraClass}"
	onclose={handleClose}
	onclick={handleDialogClick}
>
	<div bind:this={panelEl} class="m3-bottom-sheet__panel" class:m3-bottom-sheet__panel--dragging={dragging}>
		<div
			class="m3-bottom-sheet__handle-area"
			role="button"
			tabindex="0"
			aria-label="Drag to dismiss"
			onpointerdown={handlePointerDown}
			onpointermove={handlePointerMove}
			onpointerup={endDrag}
			onpointercancel={endDrag}
		>
			<div class="m3-bottom-sheet__handle"></div>
		</div>
		<div class="m3-bottom-sheet__body">
			{@render children()}
		</div>
	</div>
</dialog>

<style>
	.m3-bottom-sheet {
		margin: auto auto 0 auto;
		padding: 0;
		border: none;
		width: 100%;
		max-width: 640px;
		background: transparent;
	}

	.m3-bottom-sheet::backdrop {
		background: color-mix(in srgb, var(--md-sys-color-scrim) 48%, transparent);
		transition: background-color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
	}

	@starting-style {
		.m3-bottom-sheet::backdrop {
			background-color: transparent;
		}
	}

	.m3-bottom-sheet__panel {
		border-radius: var(--md-sys-shape-corner-extra-large) var(--md-sys-shape-corner-extra-large) 0 0;
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface);
		box-shadow: var(--md-sys-elevation-shadow-level3);
		max-height: 80vh;
		display: flex;
		flex-direction: column;
		transition: transform var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
	}

	.m3-bottom-sheet__panel--dragging {
		transition: none;
	}

	.m3-bottom-sheet__handle-area {
		display: flex;
		justify-content: center;
		padding: 12px 0;
		cursor: grab;
		touch-action: none;
		flex-shrink: 0;
	}

	.m3-bottom-sheet__panel--dragging .m3-bottom-sheet__handle-area {
		cursor: grabbing;
	}

	.m3-bottom-sheet__handle {
		width: 32px;
		height: 4px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-outline-variant);
	}

	.m3-bottom-sheet__body {
		overflow-y: auto;
		padding: 0 24px 24px;
	}
</style>
