<script lang="ts">
	import IconChevronLeft from '../icons/IconChevronLeft.svelte';
	import IconChevronRight from '../icons/IconChevronRight.svelte';
	import { pageWindow } from '$lib/pagination';

	// Page controls: previous, page numbers (with gaps), next. Give `href` for pages that live in
	// the URL, or `onselect` for a list paged in place.
	let {
		page,
		pages,
		href,
		onselect,
		label = 'Pages',
	}: {
		page: number;
		pages: number;
		href?: (page: number) => string;
		onselect?: (page: number) => void;
		label?: string;
	} = $props();

	const items = $derived(pageWindow(page, pages));
</script>

{#snippet control(target: number, text: string, current: boolean, aria?: string)}
	{@const disabled = target < 1 || target > pages}
	{#if href && !disabled && !current}
		<a class="pager__btn" href={href(target)} aria-label={aria}>{@render body(text)}</a>
	{:else}
		<button
			type="button"
			class="pager__btn"
			class:pager__btn--current={current}
			aria-label={aria}
			aria-current={current ? 'page' : undefined}
			disabled={disabled || (current && !!href)}
			onclick={() => !current && onselect?.(target)}
		>
			{@render body(text)}
		</button>
	{/if}
{/snippet}

{#snippet body(text: string)}
	{#if text === 'prev'}<IconChevronLeft size={18} />{:else if text === 'next'}<IconChevronRight size={18} />{:else}{text}{/if}
{/snippet}

{#if pages > 1}
	<nav class="pager" aria-label={label}>
		{@render control(page - 1, 'prev', false, 'Previous page')}
		<ol class="pager__pages">
			{#each items as item, i (i)}
				<li>
					{#if item === 'gap'}
						<span class="pager__gap" aria-hidden="true">…</span>
					{:else}
						{@render control(item, String(item), item === page, `Page ${item}`)}
					{/if}
				</li>
			{/each}
		</ol>
		{@render control(page + 1, 'next', false, 'Next page')}
	</nav>
{/if}

<style>
	.pager {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 4px;
		flex-wrap: wrap;
	}
	.pager__pages {
		display: flex;
		align-items: center;
		gap: 4px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.pager__btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 40px;
		height: 40px;
		padding: 0 8px;
		box-sizing: border-box;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.875rem;
		font-variant-numeric: tabular-nums;
		text-decoration: none;
		cursor: pointer;
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-fast, 150ms);
	}
	.pager__btn:hover:not(:disabled) {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
		border-radius: var(--md-sys-shape-corner-medium);
	}
	.pager__btn:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.pager__btn:disabled {
		cursor: default;
		opacity: 0.38;
	}
	.pager__btn--current,
	.pager__btn--current:disabled {
		opacity: 1;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		border-radius: var(--md-sys-shape-corner-medium);
		font-weight: 700;
	}
	.pager__gap {
		display: inline-flex;
		min-width: 24px;
		justify-content: center;
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
