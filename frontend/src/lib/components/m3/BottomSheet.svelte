<script lang="ts">
	import { m } from '$lib/paraglide/messages';
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
	// Matches --md-sys-motion-duration-short4, the duration the panel/backdrop transitions
	// below actually use — closing the native <dialog> is deliberately delayed by this long so
	// the slide-out and fade-out have time to play before the element leaves the top layer.
	const CLOSE_ANIMATION_MS = 200;

	let dialogEl: HTMLDialogElement | undefined = $state();
	let panelEl: HTMLDivElement | undefined = $state();
	let dragging = $state(false);
	let closing = $state(false);
	// The panel reached the top of the screen (small screens, tall content): it becomes a
	// full-screen sheet, and its content scrolls as one surface.
	let expanded = $state(false);

	let startY = 0;
	let panelHeight = 0;
	let activePointerId: number | undefined;

	$effect(() => {
		if (!panelEl || !open) return;
		const panel = panelEl;
		const measure = () => {
			expanded = panel.getBoundingClientRect().height >= window.innerHeight - 1;
		};
		const observer = new ResizeObserver(measure);
		observer.observe(panel);
		window.addEventListener('resize', measure);
		return () => {
			observer.disconnect();
			window.removeEventListener('resize', measure);
		};
	});

	// The page behind a modal sheet stays put: scrolling never leaks through to it.
	$effect(() => {
		if (!open) return;
		const root = document.documentElement;
		const previous = root.style.overflow;
		root.style.overflow = 'hidden';
		return () => {
			root.style.overflow = previous;
		};
	});

	$effect(() => {
		if (!dialogEl) return;
		if (open && !dialogEl.open) {
			dialogEl.showModal();
			// showModal() focuses the first focusable thing inside, the drag handle, which then
			// wears a focus ring before anyone has touched the keyboard. Start on the sheet itself:
			// Tab still reaches the handle and everything in the sheet.
			panelEl?.focus({ preventScroll: true });
		} else if (!open && dialogEl.open && !closing) {
			animateClose();
		}
	});

	// Native <dialog>.close() removes the element from the top layer immediately — there's no
	// exit-transition support built in. To let the slide-down/fade-out actually play, this holds
	// the dialog open (visually) for exactly as long as the CSS transition takes, then closes it
	// for real. Covers every path to closed: the Cancel/backdrop-click handlers below, a parent
	// setting the bindable `open` prop to false directly, and Escape (via handleCancel below).
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

	// The handle is announced as a button: Enter or Space dismisses the sheet.
	function handleHandleKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' || event.key === ' ') {
			event.preventDefault();
			open = false;
		}
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
			// Continue the drag's own inline transform smoothly to fully off-screen rather than
			// handing off to the CSS class-based closing transition, which starts from the
			// resting position and would jump the panel back to 0 first. `closing = true` up
			// front stops the $effect above from also calling animateClose() when `open` flips.
			closing = true;
			panelEl.style.transform = `translateY(100%)`;
			setTimeout(() => {
				dialogEl?.close();
				closing = false;
				open = false;
				if (panelEl) panelEl.style.transform = '';
			}, CLOSE_ANIMATION_MS);
		} else {
			panelEl.style.transform = '';
		}
	}
</script>

<dialog
	bind:this={dialogEl}
	class="m3-bottom-sheet {extraClass}"
	class:m3-bottom-sheet--closing={closing}
	onclose={handleClose}
	oncancel={handleCancel}
	onclick={handleDialogClick}
>
	<div
		bind:this={panelEl}
		class="m3-bottom-sheet__panel"
		tabindex="-1"
		class:m3-bottom-sheet__panel--dragging={dragging}
		class:m3-bottom-sheet__panel--expanded={expanded}
	>
		<div
			class="m3-bottom-sheet__handle-area"
			role="button"
			tabindex="0"
			aria-label={m.sheet_drag()}
			onkeydown={handleHandleKeydown}
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
		max-width: min(640px, 100%);
		/* Undo the UA dialog inset (max-height: calc(100% - 2em - 6px)) so a sheet can reach
		   the top of the screen. */
		max-height: 100dvh;
		overflow: hidden;
		background: transparent;
		/* showModal() moves initial focus onto the <dialog> itself (nothing inside asks for
		   autofocus), and the UA default focus ring shows on it — a plain blue rectangle around
		   the whole sheet. The dialog is a container, not a control a keyboard user interacts
		   with directly, so it isn't a meaningful focus target to indicate; the actual form
		   fields/buttons inside keep their own normal focus-visible styling untouched. */
		outline: none;
	}

	.m3-bottom-sheet::backdrop {
		background: color-mix(in srgb, var(--md-sys-color-scrim) 48%, transparent);
		transition: background-color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
	}

	.m3-bottom-sheet--closing::backdrop {
		background-color: transparent;
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
		max-height: 80dvh;
		display: flex;
		flex-direction: column;
		/* Focused on open (see the effect above) so nothing inside starts with a focus ring. */
		outline: none;
		transition: transform var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
	}

	/* Small screens: tall content opens the sheet to full screen instead of trapping it in a
	   short scrolling box. */
	@media (max-width: 600px) {
		.m3-bottom-sheet__panel {
			max-height: 100dvh;
		}
	}

	/* Full screen keeps the sheet's rounded top: it's still a sheet over the page. */
	.m3-bottom-sheet__panel--expanded {
		padding-top: env(safe-area-inset-top);
	}

	/* Entrance: the panel slides up from fully off-screen. @starting-style only governs the
	   first style change after an element enters the top layer (dialog showModal()), which is
	   exactly what's needed here — it has no effect once the dialog is already open. */
	@starting-style {
		.m3-bottom-sheet__panel {
			transform: translateY(100%);
		}
	}

	/* Exit: applied by the `closing` state (see animateClose() in the script) for every close
	   path except drag-dismiss, which continues its own inline transform instead — see endDrag(). */
	.m3-bottom-sheet--closing .m3-bottom-sheet__panel {
		transform: translateY(100%);
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
		min-height: 0;
		overflow-x: hidden;
		overflow-y: auto;
		overscroll-behavior: contain;
		padding: 0 24px calc(24px + env(safe-area-inset-bottom));
	}
</style>
