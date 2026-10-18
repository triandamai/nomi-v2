<script lang="ts">
	import ViewFrame from './ViewFrame.svelte';
	import Value from './Value.svelte';
	import type { ListView, ViewProps } from '$lib/views/types';

	let { view }: ViewProps<ListView> = $props();
	const style = $derived(view.style ?? 'bullet');
</script>

<ViewFrame title={view.title}>
	<svelte:element this={style === 'numbered' ? 'ol' : 'ul'} class="list list--{style}">
		{#each view.items as item, i (i)}
			<li class="list__item" class:list__item--done={style === 'check' && item.done}>
				<span class="list__mark" aria-hidden="true">
					{#if style === 'numbered'}{i + 1}
					{:else if style === 'check'}
						{#if item.done}<svg width="14" height="14" viewBox="0 0 24 24"><path d="M5 12.5l4.5 4.5L19 7.5" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" /></svg>{/if}
					{/if}
				</span>
				<span class="list__text">
					{#if item.url}
						<a class="list__title list__link" href={item.url} target="_blank" rel="noopener noreferrer">{item.title}</a>
					{:else}
						<span class="list__title">{item.title}</span>
					{/if}
					{#if item.detail}<span class="list__detail">{item.detail}</span>{/if}
				</span>
				{#if item.meta !== undefined && item.meta !== null}<span class="list__meta"><Value value={item.meta} format={item.format} /></span>{/if}
			</li>
		{/each}
	</svelte:element>
</ViewFrame>

<style>
	.list {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.list__item {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		padding: 10px 0;
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.list__item:first-child {
		border-top: none;
		padding-top: 0;
	}
	.list__item:last-child {
		padding-bottom: 0;
	}
	.list__mark {
		flex: none;
		display: grid;
		place-items: center;
		width: 22px;
		height: 22px;
		margin-top: 1px;
		border-radius: var(--md-sys-shape-corner-full);
		font-size: 0.75rem;
		font-weight: 700;
		font-variant-numeric: tabular-nums;
	}
	.list--bullet .list__mark::before {
		content: '';
		width: 6px;
		height: 6px;
		border-radius: 50%;
		background: var(--md-sys-color-on-surface-variant);
	}
	.list--numbered .list__mark {
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
	}
	.list--check .list__mark {
		box-shadow: inset 0 0 0 2px var(--md-sys-color-outline);
	}
	.list__item--done .list__mark {
		box-shadow: none;
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
	.list__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		flex: 1;
		min-width: 0;
	}
	.list__title {
		font-weight: 600;
		line-height: 1.4;
		overflow-wrap: anywhere;
	}
	.list__item--done .list__title {
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: line-through;
	}
	.list__link {
		color: var(--md-sys-color-primary);
		text-decoration: none;
	}
	.list__link:hover {
		text-decoration: underline;
	}
	.list__link:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
		border-radius: 4px;
	}
	.list__detail {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		line-height: 1.4;
	}
	.list__meta {
		flex: none;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
	}
</style>
