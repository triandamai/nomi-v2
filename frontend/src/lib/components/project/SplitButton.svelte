<script lang="ts">
	import type { Component } from 'svelte';
	import { ChevronDown, type LucideProps } from '@lucide/svelte';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';
	import Menu from '$lib/components/ds/overlays/Menu.svelte';
	import type { MenuEntry } from '$lib/components/ds/overlays/MenuPanel.svelte';

	// A button with a chevron beside it: the label does the main thing, the chevron opens a Menu
	// of the rest, grouped (`{ group }` entries). Two joined segments, outlined, so the bar
	// stays light. The menu is the ds Menu: same keyboard, same top-layer panel.
	type Props = {
		label: string;
		/** Names the chevron for screen readers ("More project tools"). */
		moreLabel: string;
		items: MenuEntry[];
		icon?: Component<LucideProps>;
		/** Hide the label on narrow screens, keeping the icon. */
		compact?: boolean;
		onclick?: () => void;
	};
	let { label, moreLabel, items, icon, compact = false, onclick }: Props = $props();
</script>

<span class="split">
	<button type="button" class="split__main" class:split__main--compact={compact} aria-label={compact ? label : undefined} {onclick}>
		{#if icon}<Icon {icon} size={18} />{/if}<span class="split__label">{label}</span>
	</button>
	<Menu {items} align="end" width={260}>
		{#snippet trigger(props)}
			<button type="button" class="split__more" aria-label={moreLabel} title={moreLabel} {...props}>
				<span class="split__chevron" class:split__chevron--open={props['aria-expanded']}><Icon icon={ChevronDown} size={18} /></span>
			</button>
		{/snippet}
	</Menu>
</span>

<style>
	.split {
		display: inline-flex;
		flex: none;
		align-items: stretch;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.split__main,
	.split__more {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		min-height: 40px;
		border: 0;
		background: none;
		color: inherit;
		font: inherit;
		font-size: 0.875rem;
		font-weight: 600;
		cursor: pointer;
		outline: none;
	}
	.split__main {
		padding: 0 14px 0 16px;
		border-radius: var(--md-sys-shape-corner-full) 0 0 var(--md-sys-shape-corner-full);
	}
	.split__more {
		min-width: 40px;
		padding: 0 8px 0 6px;
		border-left: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 0 var(--md-sys-shape-corner-full) var(--md-sys-shape-corner-full) 0;
	}
	.split__main:hover,
	.split__more:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.split__main:focus-visible,
	.split__more:focus-visible {
		box-shadow: inset 0 0 0 2px var(--md-sys-color-primary);
	}
	.split__chevron {
		display: inline-flex;
		transition: transform 150ms;
	}
	.split__chevron--open {
		transform: rotate(180deg);
	}
	@media (prefers-reduced-motion: reduce) {
		.split__chevron {
			transition: none;
		}
	}
	@media (max-width: 1100px) {
		.split__main--compact .split__label {
			display: none;
		}
		.split__main--compact {
			min-width: 44px;
			padding: 0 6px 0 12px;
		}
	}
	@media (pointer: coarse) {
		.split__main,
		.split__more {
			min-height: 44px;
		}
	}
</style>
