<script lang="ts">
	// Search chats, memory and files (⌘K): the person's own, through /search (GET /api/search).
	// Each hit opens where it lives ($lib/search).
	import { m } from '$lib/paraglide/messages';
	import { goto } from '$app/navigation';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import IconSearch from '$lib/components/icons/IconSearch.svelte';
	import { SEARCH_DEBOUNCE_MS, isSearchShortcut, searchHref, searchUrl } from '$lib/search';
	import { searchDialog } from '$lib/searchDialog.svelte';
	import type { SearchKind, SearchResult } from '$lib/types';

	let query = $state('');
	let results = $state<SearchResult[]>([]);
	let searched = $state('');
	let failed = $state(false);
	let input = $state<HTMLInputElement | null>(null);

	const KIND_LABEL: Record<SearchKind, () => string> = {
		chat: m.search_kind_chat,
		memory: m.search_kind_memory,
		file: m.search_kind_file,
	};

	$effect(() => {
		const onkey = (event: KeyboardEvent) => {
			if (!isSearchShortcut(event)) return;
			event.preventDefault();
			searchDialog.open = true;
		};
		window.addEventListener('keydown', onkey);
		return () => window.removeEventListener('keydown', onkey);
	});

	$effect(() => {
		if (searchDialog.open) queueMicrotask(() => input?.focus());
	});

	// Runs once typing pauses; a reply to an older query is dropped.
	$effect(() => {
		const url = searchUrl(query);
		if (!url) {
			results = [];
			searched = '';
			failed = false;
			return;
		}
		const asked = query.trim();
		const controller = new AbortController();
		const timer = setTimeout(async () => {
			try {
				const response = await fetch(url, { signal: controller.signal });
				if (!response.ok) throw new Error(String(response.status));
				const body = (await response.json()) as { results: SearchResult[] };
				results = body.results;
				failed = false;
			} catch (e) {
				if ((e as Error).name === 'AbortError') return;
				results = [];
				failed = true;
			}
			searched = asked;
		}, SEARCH_DEBOUNCE_MS);
		return () => {
			clearTimeout(timer);
			controller.abort();
		};
	});

	function open(result: SearchResult) {
		searchDialog.open = false;
		query = '';
		void goto(searchHref(result));
	}
</script>

<Dialog bind:open={searchDialog.open} class="search-dialog">
	<label class="field">
		<IconSearch size={20} />
		<input
			bind:this={input}
			bind:value={query}
			type="search"
			placeholder={m.search_placeholder()}
			aria-label={m.search_title()}
			autocomplete="off"
			onkeydown={(event) => {
				if (event.key === 'Enter' && results[0]) open(results[0]);
			}}
		/>
	</label>
	{#if failed}
		<p class="note" role="alert">{m.search_failed()}</p>
	{:else if searched && results.length === 0}
		<p class="note">{m.search_empty({ query: searched })}</p>
	{:else if results.length > 0}
		<ul class="hits">
			{#each results as result (`${result.kind}-${result.project_id ?? ''}-${result.id}`)}
				<li>
					<a
						class="hit"
						href={searchHref(result)}
						onclick={(event) => {
							event.preventDefault();
							open(result);
						}}
					>
						<span class="hit__top">
							<span class="hit__title">{result.title || m.search_untitled()}</span>
							<span class="hit__kind">{KIND_LABEL[result.kind]()}</span>
						</span>
						{#if result.snippet}<span class="hit__snippet">{result.snippet}</span>{/if}
					</a>
				</li>
			{/each}
		</ul>
	{/if}
</Dialog>

<style>
	:global(.search-dialog) {
		width: min(640px, calc(100vw - 32px));
	}
	.field {
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 48px;
		padding: 0 14px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface-variant);
	}
	.field:focus-within {
		outline: 2px solid var(--md-sys-color-primary);
	}
	.field input {
		flex: 1;
		min-width: 0;
		border: 0;
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: 1rem;
		outline: none;
	}
	.note {
		margin: 16px 4px 4px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.hits {
		max-height: min(60vh, 480px);
		margin: 12px 0 0;
		padding: 0;
		overflow-y: auto;
		list-style: none;
	}
	.hit {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-height: 44px;
		padding: 10px 12px;
		border-radius: var(--md-sys-shape-corner-medium);
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
	}
	.hit:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	.hit:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.hit__top {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
	}
	.hit__title {
		overflow: hidden;
		font-size: 0.9375rem;
		font-weight: 600;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.hit__kind {
		flex-shrink: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.75rem;
	}
	.hit__snippet {
		overflow: hidden;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
</style>
