<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { LogOut } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { page } from '$app/state';
	import { afterNavigate } from '$app/navigation';
	import DsSidebar from '$lib/components/ds/chrome/Sidebar.svelte';
	import RecentsList from '$lib/components/ds/chrome/RecentsList.svelte';
	import UsageCard from '$lib/components/ds/chrome/UsageCard.svelte';
	import UserRow from '$lib/components/ds/chrome/UserRow.svelte';
	import NavItem from '$lib/components/ds/navigation/NavItem.svelte';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';
	import { FEATURES, PERMANENT, type FeatureId } from '$lib/features';
	import { currentPins } from '$lib/drawerStore.svelte';
	import { inbox } from '$lib/notifications.svelte';
	import { NAV_ICONS } from '$lib/navIcons';
	import { isRecentChatActive, loadRecentChats, type RecentsSettings, type RecentsState } from '$lib/drawerRecents';
	import { onChatsChanged } from '$lib/userActivity';
	import { timeAgo } from '$lib/i18n';
	import { formatTokens, usageShare } from '$lib/usage';
	import type { Profile, UsageBrief } from '$lib/types';

	// The phone's navigation drawer, drawn with the design system's Sidebar (frontend-new's), over
	// the page. Unlike the desktop drawer it scrolls as one: brand, New chat, the sections, recents
	// and the person all move together, so a short screen reaches every part of it.
	let {
		open = $bindable(false),
		userEmail,
		profile,
		usage = null,
		savedPins = null,
		recents,
	}: { open?: boolean; userEmail: string; profile: Profile; usage?: UsageBrief | null; savedPins?: string[] | null; recents: RecentsSettings } = $props();

	let panel: HTMLElement | undefined = $state();
	let recentChats = $state<RecentsState>({ status: 'loading' });

	const ids = $derived<FeatureId[]>([...PERMANENT, ...currentPins(savedPins)]);
	const share = $derived(usage ? usageShare(usage.tokens_used, usage.plan.monthly_tokens) : 0);

	function isActive(href: string): boolean {
		const path = page.url.pathname;
		if (href === '/') return path === '/';
		if (href === '/chats') return path === '/chats' || path.startsWith('/chat/');
		return path === href || path.startsWith(`${href}/`);
	}

	async function refresh() {
		const next = await loadRecentChats(fetch, recents.limit);
		if (next.status === 'error' && recentChats.status === 'ready') return;
		recentChats = next;
	}

	onMount(() => {
		if (recents.enabled) void refresh();
		return onChatsChanged(() => void refresh());
	});
	// Going somewhere closes it.
	afterNavigate(() => (open = false));

	$effect(() => {
		if (!open) return;
		tick().then(() => panel?.focus());
		const onkey = (event: KeyboardEvent) => {
			if (event.key === 'Escape') open = false;
		};
		window.addEventListener('keydown', onkey);
		return () => window.removeEventListener('keydown', onkey);
	});
</script>

{#if open}
	<button type="button" class="scrim" aria-label={m.nav_close_menu()} onclick={() => (open = false)}></button>
	<div bind:this={panel} class="panel" role="dialog" aria-modal="true" aria-label={m.nav_main()} tabindex="-1">
		<DsSidebar newChatHref="/chat/new" newChatShortcut="" oncollapse={() => (open = false)}>
			{#each ids as id (id)}
				{@const feature = FEATURES[id]}
				<NavItem
					href={feature.href}
					icon={NAV_ICONS[id]}
					label={feature.label()}
					count={id === 'notifications' && inbox.unread > 0 ? inbox.unread : undefined}
					active={isActive(feature.href)}
				/>
			{/each}
			<NavItem href="/more" icon={NAV_ICONS.more} label={m.nav_more()} active={isActive('/more')} />

			{#snippet recents()}
				{#if recentChats.status === 'ready' && recentChats.chats.length > 0}
					<RecentsList
						allHref="/chats"
						items={recentChats.chats.map((chat) => ({
							href: chat.href,
							kind: 'chat' as const,
							title: chat.title,
							when: timeAgo(chat.updatedAt),
							active: isRecentChatActive(page.url.pathname, chat.id),
						}))}
					/>
				{/if}
			{/snippet}

			{#snippet footer()}
				{#if usage}
					<UsageCard
						plan={usage.plan.name}
						used={Math.min(1, share)}
						caption={m.usage_of_tokens({ used: formatTokens(usage.tokens_used), total: formatTokens(usage.plan.monthly_tokens) })}
						href="/billing"
					/>
				{/if}
				<UserRow name={profile.display_name || userEmail} avatar={profile.avatar_url ?? undefined} settingsHref="/preferences" />
				<form method="POST" action="/logout">
					<button type="submit" class="log-out"><Icon icon={LogOut} size={18} />{m.nav_log_out()}</button>
				</form>
			{/snippet}
		</DsSidebar>
	</div>
{/if}

<style>
	.scrim {
		position: fixed;
		inset: 0;
		z-index: 40;
		padding: 0;
		border: none;
		background: color-mix(in srgb, black 40%, transparent);
		cursor: default;
	}
	/* The whole drawer scrolls, top to bottom: the Sidebar twin's own layout keeps its recents
	   scrolling inside a fixed column, which on a phone leaves most of the drawer out of reach. */
	.panel {
		position: fixed;
		inset: 0 auto 0 0;
		z-index: 50;
		width: min(320px, 88vw);
		height: 100dvh;
		overflow-y: auto;
		overscroll-behavior: contain;
		background: var(--md-sys-color-surface-container-low);
		outline: none;
	}
	.panel :global(.drawer) {
		width: 100%;
		height: auto;
		min-height: 100%;
		padding-bottom: max(12px, env(safe-area-inset-bottom));
	}
	.panel :global(.recents) {
		flex: 1 0 auto;
		overflow: visible;
	}
	.log-out {
		display: flex;
		width: 100%;
		min-height: 44px;
		align-items: center;
		gap: 12px;
		padding: 0 16px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-size: 0.875rem;
		font-weight: 600;
		cursor: pointer;
	}
	.log-out:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
		color: var(--md-sys-color-on-surface);
	}
	.log-out:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
</style>
