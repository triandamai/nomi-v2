<script lang="ts">
	// Above a list: search it, and (unless `layouts` is off) show it as a table or as cards.
	import { m } from '$lib/paraglide/messages';
	import IconSearch from './icons/IconSearch.svelte';
	import IconTable from './icons/IconTable.svelte';
	import IconApps from './icons/IconApps.svelte';
	import type { ListLayout } from '$lib/listView';

	let {
		query = $bindable(''),
		layout = $bindable<ListLayout>('table'),
		placeholder,
		layouts = true,
		onlayout,
	}: { query?: string; layout?: ListLayout; placeholder: string; layouts?: boolean; onlayout?: (layout: ListLayout) => void } = $props();

	function choose(next: ListLayout) {
		layout = next;
		onlayout?.(next);
	}
</script>

<div class="toolbar">
	<label class="search">
		<IconSearch size={18} />
		<input type="search" bind:value={query} {placeholder} aria-label={placeholder} />
	</label>
	{#if layouts}
		<div class="layout" role="radiogroup" aria-label={m.list_layout()}>
			<button type="button" role="radio" aria-checked={layout === 'table'} class="layout__option" onclick={() => choose('table')} title={m.list_table()}>
				<IconTable size={18} /><span class="layout__label">{m.list_table()}</span>
			</button>
			<button type="button" role="radio" aria-checked={layout === 'cards'} class="layout__option" onclick={() => choose('cards')} title={m.list_cards()}>
				<IconApps size={18} /><span class="layout__label">{m.list_cards()}</span>
			</button>
		</div>
	{/if}
</div>

<style>
	.toolbar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px 12px;
	}
	.search {
		display: flex;
		flex: 1 1 240px;
		align-items: center;
		gap: 8px;
		max-width: 420px;
		min-height: 40px;
		padding: 0 12px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 8px;
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface-variant);
	}
	.search:focus-within {
		border-color: var(--md-sys-color-primary);
		outline: 1px solid var(--md-sys-color-primary);
	}
	.search input {
		flex: 1;
		min-width: 0;
		border: none;
		outline: none;
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: inherit;
	}
	.layout {
		display: inline-flex;
		margin-left: auto;
		padding: 2px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 8px;
	}
	.layout__option {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		min-height: 34px;
		padding: 0 10px;
		border: none;
		border-radius: 6px;
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-size: 0.875rem;
		font-weight: 600;
		cursor: pointer;
	}
	.layout__option:hover {
		color: var(--md-sys-color-on-surface);
	}
	.layout__option[aria-checked='true'] {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.layout__option:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 1px;
	}
	@media (max-width: 480px) {
		.layout__label {
			display: none;
		}
	}
</style>
