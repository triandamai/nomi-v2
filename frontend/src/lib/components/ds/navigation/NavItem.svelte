<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import { rowStyle } from './rowStyle';

	// Twin of frontend-new's NavItem: an M3 navigation-drawer item. Active = secondary-container
	// pill. The container decides how far in it sits and its icon's tone (rowStyle).
	type Props = { href: string; icon: Component<LucideProps>; label: string; count?: number | string; active?: boolean };
	let { href, icon, label, count, active = false }: Props = $props();

	const style = rowStyle();
</script>

<a {href} aria-current={active ? 'page' : undefined} class="item" class:item--dense={style.dense} class:item--quiet={style.icon === 'text-3'}>
	<span class="icon"><Icon {icon} size={20} /></span>
	<span class="label">{label}</span>
	{#if count !== undefined}<span class="count">{count}</span>{/if}
</a>

<style>
	.item {
		display: flex;
		width: 100%;
		height: 56px;
		align-items: center;
		gap: 12px;
		padding: 0 24px 0 16px;
		border-radius: var(--md-sys-shape-corner-full);
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: none;
		font: var(--md-sys-typescale-label-large-weight) var(--md-sys-typescale-label-large-size) / var(--md-sys-typescale-label-large-line-height) var(--md-sys-typescale-label-large-font);
	}
	.item--dense {
		height: 48px;
		padding: 0 16px 0 12px;
	}
	.item:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
		color: var(--md-sys-color-on-surface);
	}
	.item:focus-visible {
		outline: 3px solid var(--md-sys-color-primary);
		outline-offset: -3px;
	}
	.item[aria-current='page'] {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.icon {
		display: inline-flex;
	}
	.item--quiet:not([aria-current='page']) .icon {
		color: var(--md-sys-color-outline);
	}
	.label {
		min-width: 0;
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.count {
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
	}
</style>
