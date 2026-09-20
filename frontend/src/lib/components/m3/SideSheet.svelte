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

	// Matches --md-sys-motion-duration-short4, the duration the panel/backdrop transitions
	// below actually use — closing the native <dialog> is deliberately delayed by this long so
	// the slide-out and fade-out have time to play before the element leaves the top layer.
	const CLOSE_ANIMATION_MS = 200;

	let dialogEl: HTMLDialogElement | undefined = $state();
	let closing = $state(false);

	$effect(() => {
		if (!dialogEl) return;
		if (open && !dialogEl.open) {
			dialogEl.showModal();
		} else if (!open && dialogEl.open && !closing) {
			animateClose();
		}
	});

	// Native <dialog>.close() removes the element from the top layer immediately — there's no
	// exit-transition support built in. To let the slide-out/fade-out actually play, this holds
	// the dialog open (visually) for exactly as long as the CSS transition takes, then closes it
	// for real. Covers every path to closed: backdrop click, a parent setting the bindable
	// `open` prop to false directly, and Escape (via handleCancel below).
	function animateClose() {
		closing = true;
		setTimeout(() => {
			dialogEl?.close();
			closing = false;
		}, CLOSE_ANIMATION_MS);
	}

	function handleClose() {
		open = false;
	}

	// <dialog>'s default Escape-to-close fires `cancel` first and closes synchronously right
	// after — too fast to animate. Preventing the default here routes Escape through the same
	// `open = false` → animateClose() path every other close trigger already uses.
	function handleCancel(event: Event) {
		event.preventDefault();
		open = false;
	}

	function handleDialogClick(event: MouseEvent) {
		if (event.target === dialogEl) open = false;
	}
</script>

<dialog
	bind:this={dialogEl}
	class="m3-side-sheet {extraClass}"
	class:m3-side-sheet--closing={closing}
	onclose={handleClose}
	oncancel={handleCancel}
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

	.m3-side-sheet--closing::backdrop {
		background-color: transparent;
	}

	@starting-style {
		.m3-side-sheet::backdrop {
			background-color: transparent;
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

	/* Entrance: the panel slides in from fully off-screen. @starting-style only governs the
	   first style change after an element enters the top layer (dialog showModal()) — it must
	   target the same element the `transition` above is declared on (.m3-side-sheet__panel, not
	   the outer <dialog>) or the browser has nothing to animate between. */
	@starting-style {
		.m3-side-sheet__panel {
			transform: translateX(100%);
		}
	}

	/* Exit: applied by the `closing` state (see animateClose() in the script) for every close
	   path — backdrop click, Escape, or a parent setting `open` false directly. */
	.m3-side-sheet--closing .m3-side-sheet__panel {
		transform: translateX(100%);
	}

	.m3-side-sheet__body {
		overflow-y: auto;
		padding: 24px;
		flex: 1;
	}
</style>
