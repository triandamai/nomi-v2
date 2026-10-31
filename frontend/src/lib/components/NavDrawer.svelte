<script lang="ts">
	import { onMount, untrack } from 'svelte';
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
	import { isNavActive } from '$lib/navActive';
	import { isRecentChatActive, loadRecentChats, refreshesAfterNavigation, type RecentsSettings, type RecentsState } from '$lib/drawerRecents';
	import { onChatsChanged } from '$lib/userActivity';
	import { timeAgo } from '$lib/i18n';
	import { formatTokens, usageShare } from '$lib/usage';
	import type { Profile, UsageBrief } from '$lib/types';

	// The main navigation drawer, drawn with the design system's Sidebar (frontend-new's): New chat,
	// Home, Chats and Projects, the person's pins and More, their recent chats, then usage and the
	// person at the foot. The desktop sidebar and the phone's drawer both show it.
	let {
		userEmail,
		profile,
		usage = null,
		savedPins = null,
		recents,
		oncollapse,
	}: { userEmail: string; profile: Profile; usage?: UsageBrief | null; savedPins?: string[] | null; recents: RecentsSettings; oncollapse?: () => void } = $props();

	let recentChats = $state<RecentsState>({ status: 'loading' });

	const ids = $derived<FeatureId[]>([...PERMANENT, ...currentPins(savedPins)]);
	const share = $derived(usage ? usageShare(usage.tokens_used, usage.plan.monthly_tokens) : 0);

	async function refresh() {
		if (!recents.enabled) return;
		const next = await loadRecentChats(fetch, recents.limit);
		// Keep showing the last list when a background refresh fails, rather than an error.
		if (next.status === 'error' && recentChats.status === 'ready') return;
		recentChats = next;
	}

	onMount(() => {
		void refresh();
		// A chat moved on, here or on another of the person's devices: it goes to the top.
		return onChatsChanged(() => void refresh());
	});
	afterNavigate(({ from, to }) => {
		if (refreshesAfterNavigation(from, to)) void refresh();
	});
	// Another size picked in Preferences.
	let loadedLimit = untrack(() => recents.limit);
	$effect(() => {
		if (recents.limit === loadedLimit) return;
		loadedLimit = recents.limit;
		void refresh();
	});
</script>

<DsSidebar newChatHref="/chat/new" newChatShortcut="" {oncollapse}>
	{#each ids as id (id)}
		{@const feature = FEATURES[id]}
		<NavItem
			href={feature.href}
			icon={NAV_ICONS[id]}
			label={feature.label()}
			count={id === 'notifications' && inbox.unread > 0 ? inbox.unread : undefined}
			active={isNavActive(page.url.pathname, feature.href)}
		/>
	{/each}
	<NavItem href="/more" icon={NAV_ICONS.more} label={m.nav_more()} active={isNavActive(page.url.pathname, '/more')} />

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

<style>
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
