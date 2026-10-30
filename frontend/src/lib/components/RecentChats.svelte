<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { Pin } from '@lucide/svelte';
	import IconChevronDown from '$lib/components/icons/IconChevronDown.svelte';
	import type { RecentsState } from '$lib/drawerRecents';

	// The drawer's Recents: a collapsible list of the person's latest chats under the main
	// navigation. It only draws what it's given; DrawerRecents loads the chats.
	let {
		state,
		expanded,
		activeId = null,
		ontoggle,
		onnavigate,
		onretry,
	}: {
		state: RecentsState;
		expanded: boolean;
		activeId?: string | null;
		ontoggle: () => void;
		onnavigate?: () => void;
		onretry?: () => void;
	} = $props();

	const listId = $props.id();
</script>

<section class="recents">
	<button type="button" class="recents__heading" aria-expanded={expanded} aria-controls={listId} onclick={ontoggle}>
		<span>{m.recents_title()}</span>
		<span class="recents__chevron" class:recents__chevron--open={expanded} aria-hidden="true"><IconChevronDown size={18} /></span>
	</button>

	{#if expanded}
		<div id={listId}>
			{#if state.status === 'loading'}
				<div class="recents__loading" aria-busy="true">
					<span class="sr-only" role="status">{m.recents_loading()}</span>
					{#each [0, 1, 2] as row (row)}<span class="recents__placeholder" aria-hidden="true"></span>{/each}
				</div>
			{:else if state.status === 'error'}
				<p class="recents__note">{m.recents_error()}</p>
				<button type="button" class="recents__retry" onclick={() => onretry?.()}>{m.error_retry()}</button>
			{:else if state.chats.length === 0}
				<p class="recents__note">{m.recents_empty()}</p>
			{:else}
				<nav aria-label={m.recents_nav_label()}>
					<ul class="recents__list">
						{#each state.chats as chat (chat.id)}
							{@const active = chat.id === activeId}
							<li>
								<a href={chat.href} class="recents__item" class:recents__item--active={active} aria-current={active ? 'page' : undefined} onclick={() => onnavigate?.()}>
									<span class="recents__title">{chat.title}</span>
									{#if chat.pinned}<span class="recents__pin" title={m.drawer_pinned()}><Pin size={14} strokeWidth={1.5} aria-label={m.drawer_pinned()} role="img" /></span>{/if}
								</a>
							</li>
						{/each}
					</ul>
					<a href="/chats" class="recents__all" onclick={() => onnavigate?.()}>{m.recents_see_all()}</a>
				</nav>
			{/if}
		</div>
	{/if}
</section>

<style>
	.recents {
		display: flex;
		flex-direction: column;
		padding: 0 12px 14px;
	}
	.recents__heading {
		display: flex;
		align-items: center;
		justify-content: space-between;
		height: 44px;
		padding: 0 10px 0 18px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: 0.8125rem;
		font-weight: 650;
		cursor: pointer;
		transition: background-color var(--nomi-motion-effects-fast);
	}
	.recents__chevron {
		display: flex;
		transform: rotate(-90deg);
		transition: transform var(--nomi-motion-spatial-fast);
	}
	.recents__chevron--open {
		transform: none;
	}
	.recents__list {
		display: flex;
		flex-direction: column;
		gap: 2px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.recents__item,
	.recents__all {
		display: flex;
		align-items: center;
		height: 40px;
		padding: 0 18px;
		border-radius: var(--md-sys-shape-corner-full);
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: none;
		font-size: 0.875rem;
		transition: background-color var(--nomi-motion-effects-fast);
	}
	.recents__title {
		min-width: 0;
		flex: 1;
		overflow: hidden;
		text-overflow: ellipsis;
	}
	.recents__pin {
		display: inline-flex;
		flex-shrink: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.recents__item {
		display: flex;
		align-items: center;
		gap: 8px;
		line-height: 40px;
		overflow: hidden;
		white-space: nowrap;
		text-overflow: ellipsis;
	}
	.recents__item--active {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
		font-weight: 650;
	}
	.recents__all {
		margin-top: 2px;
		font-weight: 600;
	}
	.recents__heading:hover,
	.recents__item:hover,
	.recents__all:hover,
	.recents__retry:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 6%, transparent);
	}
	.recents__item--active:hover {
		background: var(--md-sys-color-primary-container);
	}
	.recents__heading:focus-visible,
	.recents__item:focus-visible,
	.recents__all:focus-visible,
	.recents__retry:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.recents__note {
		margin: 0;
		padding: 8px 18px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.recents__retry {
		align-self: flex-start;
		height: 40px;
		margin-left: 6px;
		padding: 0 12px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: transparent;
		color: var(--md-sys-color-primary);
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: 0.875rem;
		font-weight: 650;
		cursor: pointer;
	}
	.recents__loading {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 10px 18px;
	}
	.recents__placeholder {
		height: 12px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-highest);
	}
	.recents__placeholder:nth-child(3) {
		width: 70%;
	}
	.recents__placeholder:nth-child(4) {
		width: 50%;
	}
	/* Phones: every target is at least 44px. */
	@media (max-width: 767px) {
		.recents__item,
		.recents__all,
		.recents__retry {
			height: 44px;
			line-height: 44px;
		}
	}
</style>
