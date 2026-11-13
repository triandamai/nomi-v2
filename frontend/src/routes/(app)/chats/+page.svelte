<script lang="ts">
	// Chats: one plain list, newest first with pinned ones on top. Each row is the chat's title,
	// the start of its last message and when it was last touched. Search narrows it; pages keep
	// it short.
	import { m } from '$lib/paraglide/messages';
	import { Pin } from '@lucide/svelte';
	import Pagination from '$lib/components/m3/Pagination.svelte';
	import ListToolbar from '$lib/components/ListToolbar.svelte';
	import SessionActions from '$lib/components/SessionActions.svelte';
	import IconFolder from '$lib/components/icons/IconFolder.svelte';
	import IconChatBubble from '$lib/components/icons/IconChatBubble.svelte';
	import { timeAgo } from '$lib/i18n';
	import { previewText } from '$lib/sessionPreview';
	import { pinnedFirst } from '$lib/sessionOrder';
	import { pageCount, pageOf, searchItems } from '$lib/listView';
	import type { SessionSummary } from '$lib/types';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	/** Chats on one page. */
	const PAGE_SIZE = 20;
	let query = $state('');
	let page = $state(1);

	const found = $derived(searchItems(pinnedFirst(data.sessions), query, (s) => `${s.title ?? ''} ${s.last_message?.content ?? ''}`));
	const pages = $derived(pageCount(found.length, PAGE_SIZE));
	const shown = $derived(pageOf(found, page, PAGE_SIZE));
	// A new search starts from the first page.
	$effect(() => {
		void query;
		page = 1;
	});

	const href = (s: SessionSummary) => (s.project_id ? `/projects/session/${s.id}` : `/chat/${s.id}`);
	const title = (s: SessionSummary) => s.title ?? m.nav_new_chat();
	const projectName = (s: SessionSummary) => data.projects.find((project) => project.id === s.in_project_id)?.name;
	const kind = (s: SessionSummary) => projectName(s) ?? (s.project_id ? m.session_project() : null);
</script>

<div class="list-page">
	<div class="list-page__body">
		<header class="list-page__head">
			<h1 class="md-headline-medium list-page__title">{m.chats_title()}</h1>
			{#if data.sessions.length > 0}<p class="list-page__count">{m.chats_count({ count: found.length })}</p>{/if}
		</header>

		{#if data.sessions.length === 0}
			<p class="list-page__empty">{m.chats_empty()}</p>
		{:else}
			<ListToolbar bind:query layouts={false} placeholder={m.chats_search()} />

			{#if found.length === 0}
				<p class="list-page__empty">{m.list_no_match({ query })}</p>
			{:else}
				<ul class="chats">
					{#each shown as session (session.id)}
						<li class="chat">
							<a class="chat__link" href={href(session)} data-sveltekit-reload={session.project_id ? true : undefined}>
								<span class="chat__icon" aria-hidden="true">
									{#if session.project_id}<IconFolder size={18} />{:else}<IconChatBubble size={18} />{/if}
								</span>
								<span class="chat__text">
									<span class="chat__top">
										<span class="chat__title">{title(session)}</span>
										{#if session.pinned}<span class="chat__pin"><Pin size={14} strokeWidth={1.5} role="img" aria-label={m.drawer_pinned()} /></span>{/if}
										<span class="chat__time">{timeAgo(session.updated_at)}</span>
									</span>
									<span class="chat__preview">
										{#if kind(session)}<span class="chat__kind">{kind(session)}</span> · {/if}{previewText(session.last_message?.content)}
									</span>
								</span>
							</a>
							<span class="chat__menu"><SessionActions {session} projects={data.projects} /></span>
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
</div>

<style>
	.list-page {
		height: 100%;
		overflow-y: auto;
		padding: 24px clamp(14px, 4vw, 32px) 44px;
		box-sizing: border-box;
	}
	.list-page__body {
		display: flex;
		flex-direction: column;
		gap: 16px;
		max-width: 760px;
		margin: 0 auto;
	}
	.list-page__head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
	}
	.list-page__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.list-page__count,
	.list-page__empty {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.list-page__pager {
		display: flex;
		justify-content: center;
	}
	.chats {
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.chat {
		display: flex;
		align-items: center;
		gap: 4px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.chat:last-child {
		border-bottom: none;
	}
	.chat__link {
		display: flex;
		flex: 1;
		align-items: center;
		gap: 12px;
		min-width: 0;
		min-height: 60px;
		padding: 10px 8px;
		border-radius: var(--md-sys-shape-corner-small);
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
	}
	.chat__link:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 6%, transparent);
	}
	.chat__link:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.chat__icon {
		display: inline-flex;
		flex: none;
		color: var(--md-sys-color-on-surface-variant);
	}
	.chat__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.chat__top {
		display: flex;
		align-items: baseline;
		gap: 8px;
		min-width: 0;
	}
	.chat__title {
		overflow: hidden;
		font-weight: 600;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.chat__pin {
		display: inline-flex;
		flex: none;
		align-self: center;
		color: var(--md-sys-color-on-surface-variant);
	}
	.chat__time {
		flex: none;
		margin-left: auto;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		font-variant-numeric: tabular-nums;
	}
	.chat__preview {
		overflow: hidden;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.chat__kind {
		color: var(--md-sys-color-on-surface);
	}
	.chat__menu {
		flex: none;
	}
</style>
