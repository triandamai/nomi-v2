<script lang="ts">
	import { onMount } from 'svelte';
	import { PanelLeftOpen, Plus } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { page } from '$app/state';
	import Avatar from '$lib/components/ds/identity/Avatar.svelte';
	import Mascot from '$lib/components/ds/agents/Mascot.svelte';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';
	import IconButton from '$lib/components/ds/actions/IconButton.svelte';
	import Tooltip from '$lib/components/ds/overlays/Tooltip.svelte';
	import NavDrawer from '$lib/components/NavDrawer.svelte';
	import SettingsDrawer from '$lib/components/SettingsDrawer.svelte';
	import { persistCollapsed, readInitialCollapsed } from '$lib/components/m3/sidebarCollapse';
	import { FEATURES, PERMANENT, type FeatureId } from '$lib/features';
	import { currentPins } from '$lib/drawerStore.svelte';
	import { inbox, badgeLabel } from '$lib/notifications.svelte';
	import { NAV_ICONS } from '$lib/navIcons';
	import { isNavActive } from '$lib/navActive';
	import { activeSetting } from '$lib/settingsNav';
	import type { RecentsSettings } from '$lib/drawerRecents';
	import type { Profile, UsageBrief } from '$lib/types';

	// The desktop sidebar: the design system's drawer (NavDrawer), or on an account page its
	// settings drawer. Collapsed, it's an icon rail of the same places; phones get MobileNavDrawer.
	let {
		userEmail,
		profile,
		usage = null,
		savedPins = null,
		recents,
	}: { userEmail: string; profile: Profile; usage?: UsageBrief | null; savedPins?: string[] | null; recents: RecentsSettings } = $props();

	const STORAGE_KEY = 'nomi:user-sidebar-collapsed';
	let collapsed = $state(false);

	onMount(() => {
		collapsed = readInitialCollapsed(STORAGE_KEY);
	});

	function toggleCollapsed() {
		collapsed = !collapsed;
		persistCollapsed(STORAGE_KEY, collapsed);
	}

	const inSettings = $derived(activeSetting(page.url.pathname) !== undefined);
	const ids = $derived<FeatureId[]>([...PERMANENT, ...currentPins(savedPins)]);
	const rail = $derived([
		...ids.map((id) => ({ href: FEATURES[id].href, label: FEATURES[id].label(), icon: NAV_ICONS[id], badge: id === 'notifications' ? inbox.unread : 0 })),
		{ href: '/more', label: m.nav_more(), icon: NAV_ICONS.more, badge: 0 },
	]);
	const accountLabel = $derived(profile.display_name || userEmail);
</script>

<aside class="sidebar" class:sidebar--rail={collapsed && !inSettings}>
	{#if inSettings}
		<div class="scroll"><SettingsDrawer {userEmail} {profile} /></div>
	{:else if !collapsed}
		<div class="scroll"><NavDrawer {userEmail} {profile} {usage} {savedPins} {recents} oncollapse={toggleCollapsed} /></div>
	{:else}
		<nav class="rail" aria-label={m.nav_main()}>
			<a href="/" class="rail__brand" aria-label={m.nav_nomi_home()}><Mascot agent="nomi" size={32} /></a>
			<Tooltip label={m.nav_new_chat()}>
				{#snippet children(props)}
					<a href="/chat/new" class="rail__new" aria-label={m.nav_new_chat()} {...props}><Icon icon={Plus} size={20} /></a>
				{/snippet}
			</Tooltip>
			<div class="rail__items">
				{#each rail as item (item.href)}
					{@const active = isNavActive(page.url.pathname, item.href)}
					<Tooltip label={item.label}>
						{#snippet children(props)}
							<a href={item.href} class="rail__item" class:rail__item--active={active} aria-current={active ? 'page' : undefined} aria-label={item.label} {...props}>
								<Icon icon={item.icon} size={20} />
								{#if item.badge > 0}<span class="rail__badge" aria-hidden="true">{badgeLabel(item.badge)}</span>{/if}
							</a>
						{/snippet}
					</Tooltip>
				{/each}
			</div>
			<IconButton icon={PanelLeftOpen} label={m.nav_expand()} variant="ghost" onclick={toggleCollapsed} />
			<a href="/preferences" class="rail__person" aria-label={m.nav_settings()}><Avatar name={accountLabel} src={profile.avatar_url ?? undefined} /></a>
		</nav>
	{/if}
</aside>

<style>
	/* Desktop only: below md the phone's drawer takes over. */
	.sidebar {
		display: none;
	}
	@media (min-width: 768px) {
		.sidebar {
			display: flex;
			height: 100%;
			flex-shrink: 0;
		}
	}
	/* The whole drawer scrolls, top to bottom, as the phone's does: the Sidebar twin's own layout
	   scrolls only its recents inside a fixed column, which on a shorter window shows a sliver. */
	.scroll {
		height: 100%;
		overflow-y: auto;
		overscroll-behavior: contain;
		scrollbar-width: thin;
		background: var(--md-sys-color-surface-container-low);
	}
	.scroll :global(.drawer) {
		height: auto;
		min-height: 100%;
	}
	.scroll :global(.recents) {
		flex: 1 0 auto;
		overflow: visible;
	}
	.rail {
		display: flex;
		width: 80px;
		height: 100%;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding: 16px 0 12px;
		background: var(--md-sys-color-surface-container-low);
	}
	.rail__brand,
	.rail__new,
	.rail__item,
	.rail__person {
		position: relative;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: none;
	}
	.rail__brand {
		width: 48px;
		height: 48px;
		border-radius: var(--md-sys-shape-corner-full);
	}
	.rail__new {
		width: 56px;
		height: 56px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.rail__items {
		display: flex;
		min-height: 0;
		flex: 1;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		overflow-y: auto;
		scrollbar-width: none;
	}
	.rail__item {
		width: 56px;
		height: 32px;
		flex-shrink: 0;
		margin: 4px 0;
		border-radius: var(--md-sys-shape-corner-full);
	}
	.rail__item--active {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.rail__badge {
		position: absolute;
		top: -4px;
		right: 6px;
		min-width: 16px;
		height: 16px;
		padding: 0 4px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-error);
		color: var(--md-sys-color-on-error);
		font-size: 0.6875rem;
		font-weight: 700;
		line-height: 16px;
		text-align: center;
	}
	.rail__person {
		border-radius: var(--md-sys-shape-corner-full);
	}
	.rail__brand:hover,
	.rail__item:hover,
	.rail__person:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.rail__item--active:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-secondary-container) 8%, var(--md-sys-color-secondary-container));
	}
	.rail__new:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-primary-container) 8%, var(--md-sys-color-primary-container));
	}
	.rail__brand:focus-visible,
	.rail__new:focus-visible,
	.rail__item:focus-visible,
	.rail__person:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
</style>
