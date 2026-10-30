<script lang="ts">
	import Icon from '../foundations/Icon.svelte';
	import type { MenuItem } from './MenuPanel.svelte';

	// An M3 menu item: icon, label, shortcut or meta; danger in the error role. `hover` shows the
	// hover without a pointer (the gallery). Lives in a role="menu". `wide` rows are taller.
	type Props = { item: MenuItem; wide?: boolean; onpick?: (item: MenuItem) => void };
	let { item, wide = false, onpick }: Props = $props();
</script>

<button
	type="button"
	role="menuitem"
	tabindex="-1"
	data-hover={item.hover || undefined}
	onclick={() => onpick?.(item)}
	class="row"
	class:row--wide={wide}
	class:row--danger={item.danger}
>
	{#if item.icon}<span class="icon"><Icon icon={item.icon} size={20} /></span>{/if}
	<span class="label">{item.label}</span>
	{#if item.meta}<span class="trailing">{item.meta}</span>{/if}
	{#if item.shortcut}<span class="trailing">{item.shortcut}</span>{/if}
</button>

<style>
	.row {
		display: flex;
		width: 100%;
		height: 48px;
		align-items: center;
		gap: 12px;
		padding: 0 12px;
		border: 0;
		background: none;
		text-align: left;
		cursor: pointer;
		outline: none;
		color: var(--md-sys-color-on-surface);
		font: var(--md-sys-typescale-label-large-weight) var(--md-sys-typescale-label-large-size) / var(--md-sys-typescale-label-large-line-height) var(--md-sys-typescale-label-large-font);
	}
	.row--wide {
		height: 56px;
	}
	.row:hover,
	.row:focus-visible,
	.row[data-hover] {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.row:focus-visible {
		box-shadow: inset 0 0 0 2px var(--md-sys-color-primary);
	}
	.row--danger {
		color: var(--md-sys-color-error);
	}
	.icon {
		display: inline-flex;
		color: var(--md-sys-color-on-surface-variant);
	}
	.row--danger .icon {
		color: var(--md-sys-color-error);
	}
	.label {
		min-width: 0;
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.trailing {
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
