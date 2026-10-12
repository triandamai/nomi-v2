<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { onMount } from 'svelte';
	import DataTable from '$lib/components/m3/DataTable.svelte';
	import Pagination from '$lib/components/m3/Pagination.svelte';
	import ListToolbar from '$lib/components/ListToolbar.svelte';
	import SessionActions from '$lib/components/SessionActions.svelte';
	import IconFolder from '$lib/components/icons/IconFolder.svelte';
	import IconChatBubble from '$lib/components/icons/IconChatBubble.svelte';
	import { timeAgo } from '$lib/i18n';
	import { previewText } from '$lib/sessionPreview';
	import { pageCount, pageOf, searchItems, storedLayout, storeLayout, type ListLayout } from '$lib/listView';
	import type { SessionSummary } from '$lib/types';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const PAGE_SIZE = 12;
	let query = $state('');
	let page = $state(1);
	let layout = $state<ListLayout>('table');
	onMount(() => (layout = storedLayout('chats', 'table')));

	const found = $derived(searchItems(data.sessions, query, (s) => `${s.title ?? ''} ${s.last_message?.content ?? ''}`));
	const pages = $derived(pageCount(found.length, PAGE_SIZE));
	const shown = $derived(pageOf(found, page, PAGE_SIZE));
	// A new search starts from the first page.
	$effect(() => {
		void query;
		page = 1;
	});

	const columns = [
		{ key: 'chat', label: m.list_col_chat() },
		{ key: 'last', label: m.list_col_last_message() },
		{ key: 'updated', label: m.list_col_updated() },
		{ key: 'actions', label: '' },
	];

	const href = (s: SessionSummary) => (s.project_id ? `/projects/session/${s.id}` : `/chat/${s.id}`);
	const title = (s: SessionSummary) => s.title ?? m.nav_new_chat();
</script>

<div class="list-page">
	<header>
		<h1 class="md-display-small list-page__title">{m.chats_title()}</h1>
	</header>

	{#if data.sessions.length === 0}
		<p class="list-page__empty">{m.chats_empty()}</p>
	{:else}
		<ListToolbar bind:query bind:layout placeholder={m.chats_search()} onlayout={(next) => storeLayout('chats', next)} />

		{#if found.length === 0}
			<p class="list-page__empty">{m.list_no_match({ query })}</p>
		{:else if layout === 'table'}
			<DataTable card {columns}>
				{#each shown as session (session.id)}
					<tr>
						<td>
							<a class="cell-title" href={href(session)} data-sveltekit-reload={session.project_id ? true : undefined}>
								<span class="kind" class:kind--project={!!session.project_id} title={session.project_id ? m.session_project() : m.session_chat()}>
									{#if session.project_id}<IconFolder size={16} />{:else}<IconChatBubble size={16} />{/if}
								</span>
								<span class="cell-title__text">{title(session)}</span>
							</a>
						</td>
						<td class="cell-preview">{previewText(session.last_message?.content)}</td>
						<td class="cell-time">{timeAgo(session.updated_at)}</td>
						<td class="cell-actions"><SessionActions {session} /></td>
					</tr>
				{/each}
				{#snippet list()}
					{#each shown as session (session.id)}
						<div role="listitem" class="row-item">
							<a class="row-item__link" href={href(session)} data-sveltekit-reload={session.project_id ? true : undefined}>
								<span class="row-item__title">{title(session)}</span>
								<span class="row-item__meta">
									{session.project_id ? m.session_project() : m.session_chat()} · {timeAgo(session.updated_at)}
								</span>
								<span class="row-item__preview">{previewText(session.last_message?.content)}</span>
							</a>
							<SessionActions {session} />
						</div>
					{/each}
				{/snippet}
			</DataTable>
		{:else}
			<ul class="cards">
				{#each shown as session (session.id)}
					<li class="card">
						<a class="card__link" href={href(session)} data-sveltekit-reload={session.project_id ? true : undefined}>
							<span class="card__title">{title(session)}</span>
						</a>
						<p class="card__preview">{previewText(session.last_message?.content)}</p>
						<p class="card__meta">
							{#if session.project_id}<span class="card__kind"><IconFolder size={14} />{m.session_project()}</span>{/if}
							<span class="card__time">{timeAgo(session.updated_at)}</span>
						</p>
						<div class="card__menu"><SessionActions {session} /></div>
					</li>
				{/each}
			</ul>
		{/if}

		{#if pages > 1}
			<div class="list-page__pager">
				<Pagination {page} {pages} onselect={(next) => (page = next)} label={m.page_pages()} />
			</div>
		{/if}
	{/if}
</div>

<style>
	.list-page {
		display: flex;
		flex-direction: column;
		gap: 16px;
		height: 100%;
		overflow-y: auto;
		padding: 24px clamp(14px, 4vw, 32px) 44px;
		box-sizing: border-box;
	}
	.list-page__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.list-page__empty {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.list-page__pager {
		display: flex;
		justify-content: center;
	}
	.cell-title {
		display: flex;
		align-items: center;
		gap: 10px;
		min-width: 0;
		max-width: 300px;
		color: var(--md-sys-color-on-surface);
		font-weight: 600;
		text-decoration: none;
	}
	.cell-title:hover .cell-title__text {
		text-decoration: underline;
		text-underline-offset: 3px;
	}
	.cell-title__text {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.kind {
		display: grid;
		flex: none;
		place-items: center;
		width: 28px;
		height: 28px;
		border-radius: 6px;
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
	}
	.kind--project {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	/* Takes the room that's left and ellipsizes, so the menu column always fits. */
	.cell-preview {
		width: 50%;
		max-width: 0;
		overflow: hidden;
		color: var(--md-sys-color-on-surface-variant);
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.cell-time {
		color: var(--md-sys-color-on-surface-variant);
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}
	.cell-actions {
		width: 48px;
		text-align: right;
	}
	.row-item {
		display: flex;
		align-items: center;
		gap: 4px;
		padding: 10px 4px 10px 14px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.row-item__link {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
	}
	.row-item__title {
		overflow: hidden;
		font-weight: 600;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.row-item__meta,
	.row-item__preview {
		overflow: hidden;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.cards {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
		gap: 12px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.card {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: 6px;
		min-height: 140px;
		padding: 14px 14px 12px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface);
		transition: background-color 150ms ease-out;
	}
	.card:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	.card:has(.card__link:focus-visible) {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.card__link {
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
		outline: none;
	}
	/* The whole card opens the chat; the menu sits above this. */
	.card__link::after {
		content: '';
		position: absolute;
		inset: 0;
		border-radius: inherit;
	}
	.card__title {
		display: -webkit-box;
		overflow: hidden;
		padding-right: 28px;
		font-size: 0.9375rem;
		font-weight: 650;
		line-height: 1.35;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
	}
	.card__preview {
		display: -webkit-box;
		overflow: hidden;
		margin: 0;
		font-size: 0.875rem;
		line-height: 1.45;
		color: var(--md-sys-color-on-surface-variant);
		-webkit-line-clamp: 3;
		line-clamp: 3;
		-webkit-box-orient: vertical;
	}
	.card__meta {
		display: flex;
		align-items: center;
		gap: 12px;
		margin: auto 0 0;
		padding-top: 6px;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.card__kind {
		display: inline-flex;
		align-items: center;
		gap: 5px;
	}
	.card__time {
		margin-left: auto;
		font-variant-numeric: tabular-nums;
	}
	.card__menu {
		position: absolute;
		z-index: 1;
		top: 8px;
		right: 6px;
	}
</style>
