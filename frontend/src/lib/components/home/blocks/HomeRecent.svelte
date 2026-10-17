<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import { timeAgo } from '$lib/i18n';
	import type { SessionSummary } from '$lib/types';

	// Recent chats: a grid of cards, or rows in one panel.

	let { sessions, style = 'cards', limit = 6 }: { sessions: SessionSummary[]; style?: 'cards' | 'list'; limit?: number } = $props();

	const shown = $derived(sessions.slice(0, limit));
	const href = (session: SessionSummary) => (session.project_id ? `/projects/session/${session.id}` : `/chat/${session.id}`);
</script>

<section class="recent" aria-labelledby="recent-heading">
	<div class="recent__head">
		<h2 id="recent-heading" class="md-title-large recent__title">{m.home_recent()}</h2>
		<a href="/chats" class="recent__all">{m.home_all_chats()}</a>
	</div>
	{#if shown.length === 0}
		<p class="md-body-medium recent__empty">{m.home_recent_empty()}</p>
	{:else if style === 'cards'}
		<ul class="recent__grid">
			{#each shown as session (session.id)}
				<li>
					<a class="recent__card" href={href(session)} data-sveltekit-reload={session.project_id ? true : undefined}>
						<span class="recent__top">
							<AgentShape size={28} working={session.agent_active} />
							<span class="nomi-meta">{session.agent_active ? m.home_working() : timeAgo(session.updated_at)}</span>
						</span>
						<span class="recent__name">{session.title ?? m.nav_new_chat()}</span>
						<span class="recent__preview">{session.last_message?.content ?? m.session_no_messages()}</span>
					</a>
				</li>
			{/each}
		</ul>
	{:else}
		<ul class="recent__list">
			{#each shown as session (session.id)}
				<li>
					<a class="recent__row" href={href(session)} data-sveltekit-reload={session.project_id ? true : undefined}>
						<AgentShape size={32} working={session.agent_active} />
						<span class="recent__text">
							<span class="recent__name">{session.title ?? m.nav_new_chat()}</span>
							<span class="recent__line">{session.last_message?.content ?? m.session_no_messages()}</span>
						</span>
						<span class="nomi-meta recent__when">{session.agent_active ? m.home_working() : timeAgo(session.updated_at)}</span>
					</a>
				</li>
			{/each}
		</ul>
	{/if}
</section>

<style>
	.recent__head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		margin-bottom: 14px;
	}
	.recent__title {
		margin: 0;
		font-weight: 700;
	}
	.recent__empty {
		color: var(--md-sys-color-on-surface-variant);
	}
	.recent__all {
		color: var(--md-sys-color-primary);
		font-weight: 600;
		text-decoration: none;
		padding: 8px 4px;
	}
	.recent__all:hover {
		text-decoration: underline;
	}
	.recent__all:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
		border-radius: 6px;
	}
	.recent__grid {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(min(100%, 260px), 1fr));
		gap: 14px;
	}
	.recent__card {
		display: flex;
		flex-direction: column;
		gap: 8px;
		height: 100%;
		box-sizing: border-box;
		padding: 16px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-fast);
	}
	.recent__card:hover {
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.recent__top {
		display: flex;
		align-items: center;
		justify-content: space-between;
	}
	.recent__name {
		font-weight: 650;
		font-size: 0.9375rem;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.recent__preview {
		font-size: 0.875rem;
		opacity: 0.8;
		display: -webkit-box;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		overflow: hidden;
	}

	/* List: rows in one translucent panel (it sits on the glass stage). */
	.recent__list {
		list-style: none;
		margin: 0;
		padding: 6px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: color-mix(in srgb, var(--md-sys-color-surface-container-lowest) 76%, transparent);
	}
	.recent__row {
		display: flex;
		align-items: center;
		gap: 14px;
		min-height: 56px;
		padding: 8px 14px 8px 10px;
		border-radius: var(--md-sys-shape-corner-large);
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
		transition: background-color var(--nomi-motion-effects-fast);
	}
	.recent__row:hover {
		background: color-mix(in srgb, var(--md-sys-color-primary-container) 70%, transparent);
	}
	.recent__card:focus-visible,
	.recent__row:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.recent__text {
		display: flex;
		flex-direction: column;
		flex: 1;
		min-width: 0;
	}
	.recent__line {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.recent__when {
		flex: none;
		text-transform: none;
	}
</style>
