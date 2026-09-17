<!-- frontend/src/lib/components/m3/SideSheet.svelte -->
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

	let dialogEl: HTMLDialogElement | undefined = $state();

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
</script>

<dialog
	bind:this={dialogEl}
	class="m3-side-sheet {extraClass}"
	onclose={handleClose}
	onclick={handleDialogClick}
>
	<div class="m3-side-sheet__panel">
		<div class="m3-side-sheet__body">
			{@render children()}
		</div>
	</div>
</dialog>

<style>
	.m3-side-sheet {
		margin: 0 0 0 auto;
		padding: 0;
		border: none;
		width: 100%;
		max-width: 420px;
		height: 100%;
		max-height: 100%;
		background: transparent;
	}

	.m3-side-sheet::backdrop {
		background: color-mix(in srgb, var(--md-sys-color-scrim) 48%, transparent);
		transition: background-color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
	}

	@starting-style {
		.m3-side-sheet::backdrop {
			background-color: transparent;
		}
		.m3-side-sheet {
			transform: translateX(100%);
		}
	}

	.m3-side-sheet__panel {
		height: 100%;
		border-radius: var(--md-sys-shape-corner-extra-large) 0 0 var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface);
		box-shadow: var(--md-sys-elevation-shadow-level3);
		display: flex;
		flex-direction: column;
		transition: transform var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
	}

	.m3-side-sheet__body {
		overflow-y: auto;
		padding: 24px;
		flex: 1;
	}
</style>
