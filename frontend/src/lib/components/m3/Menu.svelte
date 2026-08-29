<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		open = $bindable(false),
		trigger,
		children,
		class: extraClass = '',
	}: {
		open?: boolean;
		trigger: Snippet<[{ toggle: () => void }]>;
		children: Snippet;
		class?: string;
	} = $props();

	let panelEl: HTMLDivElement | undefined = $state();
	let anchorEl: HTMLDivElement | undefined = $state();

	function toggle() {
		open = !open;
	}

	const VIEWPORT_MARGIN = 8;

	// Popover elements are `display: none` (and so have a zero-size rect) until shown, so the
	// panel must be shown first — synchronously, before paint — for its real content-driven
	// width/height to be measurable at all. Position is then computed against that real size,
	// checking every edge, before the user ever sees it at the wrong spot.
	function positionPanel() {
		if (!panelEl || !anchorEl) return;
		const anchorRect = anchorEl.getBoundingClientRect();
		const panelRect = panelEl.getBoundingClientRect();
		const viewportWidth = window.innerWidth;
		const viewportHeight = window.innerHeight;

		// Vertical: prefer opening below the anchor; flip above it if that would overflow the
		// bottom edge and there's room above; otherwise clamp fully inside the viewport.
		let top = anchorRect.bottom + 4;
		if (top + panelRect.height > viewportHeight - VIEWPORT_MARGIN) {
			const above = anchorRect.top - 4 - panelRect.height;
			top = above >= VIEWPORT_MARGIN ? above : Math.max(VIEWPORT_MARGIN, viewportHeight - panelRect.height - VIEWPORT_MARGIN);
		}

		// Horizontal: prefer right-aligned to the anchor's right edge; slide left if that would
		// overflow the left edge, right if it would overflow the right edge.
		let right = viewportWidth - anchorRect.right;
		if (viewportWidth - right - panelRect.width < VIEWPORT_MARGIN) {
			right = viewportWidth - panelRect.width - VIEWPORT_MARGIN;
		}
		if (right < VIEWPORT_MARGIN) {
			right = VIEWPORT_MARGIN;
		}

		panelEl.style.top = `${top}px`;
		panelEl.style.right = `${right}px`;
	}

	$effect(() => {
		if (!panelEl) return;
		if (open) {
			if (!panelEl.matches(':popover-open')) panelEl.showPopover();
			positionPanel();
			const first = panelEl.querySelector<HTMLElement>('[role="menuitem"]');
			first?.focus();
		} else if (panelEl.matches(':popover-open')) {
			panelEl.hidePopover();
		}
	});

	function handleToggle(event: Event) {
		// Fires on every popover state change, including native light-dismiss (outside
		// click, Escape) — this is what keeps `open` in sync when the browser closes the
		// popover without Menu's own code asking it to.
		const toggleEvent = event as ToggleEvent;
		if (toggleEvent.newState === 'closed') open = false;
	}

	function handleKeydown(event: KeyboardEvent) {
		const target = event.target as HTMLElement;
		if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA') return;
		const items = Array.from(panelEl?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? []);
		if (items.length === 0) return;
		const currentIndex = items.indexOf(document.activeElement as HTMLElement);
		if (event.key === 'ArrowDown') {
			event.preventDefault();
			items[(currentIndex + 1) % items.length]?.focus();
		} else if (event.key === 'ArrowUp') {
			event.preventDefault();
			items[(currentIndex - 1 + items.length) % items.length]?.focus();
		} else if (event.key === 'Home') {
			event.preventDefault();
			items[0]?.focus();
		} else if (event.key === 'End') {
			event.preventDefault();
			items[items.length - 1]?.focus();
		}
	}
</script>

<div class="m3-menu-anchor {extraClass}" bind:this={anchorEl}>
	{@render trigger({ toggle })}
	<div
		bind:this={panelEl}
		popover="auto"
		role="menu"
		class="m3-menu-panel"
		ontoggle={handleToggle}
		onkeydown={handleKeydown}
	>
		{@render children()}
	</div>
</div>

<style>
	.m3-menu-anchor {
		position: relative;
		display: inline-block;
	}

	.m3-menu-panel {
		inset: auto;
		margin: 0;
		padding: 8px;
		min-width: 200px;
		max-width: 320px;
		max-height: 60vh;
		overflow-y: auto;
		border: none;
		border-radius: var(--md-sys-shape-corner-extra-small);
		background: var(--md-sys-color-surface-container);
		color: var(--md-sys-color-on-surface);
		box-shadow: var(--md-sys-elevation-shadow-level2);
	}
</style>
