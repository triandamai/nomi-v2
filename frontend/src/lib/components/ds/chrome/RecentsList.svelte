<script lang="ts">
	import { ArrowRight, ChevronDown } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import RecentRow from '../navigation/RecentRow.svelte';

	// The drawer's recent chats and projects, foldable, ending in a link to all chats.
	type Item = { href: string; kind: 'chat' | 'project'; title: string; when: string; active?: boolean };
	let { items, allHref, open = $bindable(true) }: { items: Item[]; allHref: string; open?: boolean } = $props();

	const id = $props.id();
</script>

<div class="recents">
	<button type="button" aria-expanded={open} aria-controls={id} onclick={() => (open = !open)} class="fold">
		<span class="nomi-meta">{m.nav_recents()}</span>
		<span class="chevron" class:chevron--closed={!open}><Icon icon={ChevronDown} size={18} /></span>
	</button>
	{#if open}
		<div {id} class="rows">
			{#each items as item (item.href)}<RecentRow {...item} />{/each}
		</div>
	{/if}
	<!-- Folded or not, this stays: it's the way to every chat. -->
	<a href={allHref} class="all">
		{m.nav_all_chats()}
		<Icon icon={ArrowRight} size={18} />
	</a>
</div>

<style>
	.recents,
	.rows {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.fold {
		display: flex;
		width: 100%;
		min-height: 40px;
		align-items: center;
		justify-content: space-between;
		padding: 0 16px;
		border: 0;
		border-radius: var(--md-sys-shape-corner-full);
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		cursor: pointer;
	}
	.fold:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.fold:focus-visible,
	.all:focus-visible {
		outline: 3px solid var(--md-sys-color-primary);
		outline-offset: -3px;
	}
	.chevron {
		display: inline-flex;
		transition: transform var(--nomi-motion-effects-fast);
	}
	.chevron--closed {
		transform: rotate(-90deg);
	}
	.all {
		display: inline-flex;
		min-height: 44px;
		align-items: center;
		gap: 6px;
		padding: 0 16px;
		border-radius: var(--md-sys-shape-corner-full);
		color: var(--md-sys-color-primary);
		text-decoration: none;
		font: var(--md-sys-typescale-label-large-weight) var(--md-sys-typescale-label-large-size) / var(--md-sys-typescale-label-large-line-height) var(--md-sys-typescale-label-large-font);
	}
	.all:hover {
		background: color-mix(in srgb, var(--md-sys-color-primary) 8%, transparent);
	}
</style>
