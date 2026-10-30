<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Link from '../actions/Link.svelte';

	// Nothing here yet, said plainly, with the one thing to do about it.
	type Props = { icon: Component<LucideProps>; title: string; count?: number; body: string; action?: { label: string; href: string } };
	let { icon, title, count, body, action }: Props = $props();
</script>

<section class="empty">
	<header class="head">
		<span class="icon"><Icon {icon} size={20} /></span>
		<span class="title">{title}</span>
		{#if count !== undefined}<span class="count">{count}</span>{/if}
	</header>
	<p class="body">{body}</p>
	{#if action}<Link href={action.href} label={action.label} />{/if}
</section>

<style>
	.empty {
		display: flex;
		width: 100%;
		flex-direction: column;
		align-items: flex-start;
		gap: 8px;
		padding: 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.head {
		display: flex;
		width: 100%;
		align-items: center;
		gap: 8px;
	}
	.icon {
		display: inline-flex;
		color: var(--md-sys-color-on-surface-variant);
	}
	.title {
		min-width: 0;
		flex: 1;
		font: var(--md-sys-typescale-title-small-weight) var(--md-sys-typescale-title-small-size) / var(--md-sys-typescale-title-small-line-height) var(--md-sys-typescale-title-small-font);
	}
	.count {
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
	}
	.body {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font: var(--md-sys-typescale-body-medium-weight) var(--md-sys-typescale-body-medium-size) / var(--md-sys-typescale-body-medium-line-height) var(--md-sys-typescale-body-medium-font);
	}
</style>
