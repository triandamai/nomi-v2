<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { enhance } from '$app/forms';
	import { page } from '$app/state';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Avatar from '$lib/components/m3/Avatar.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import IconChatBubble from '$lib/components/icons/IconChatBubble.svelte';
	import IconChevronLeft from '$lib/components/icons/IconChevronLeft.svelte';
	import IconChevronRight from '$lib/components/icons/IconChevronRight.svelte';
	import IconClose from '$lib/components/icons/IconClose.svelte';
	import IconFolder from '$lib/components/icons/IconFolder.svelte';
	import IconHome from '$lib/components/icons/IconHome.svelte';
	import IconMemory from '$lib/components/icons/IconMemory.svelte';
	import IconBell from '$lib/components/icons/IconBell.svelte';
	import IconWallet from '$lib/components/icons/IconWallet.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import Menu from '$lib/components/m3/Menu.svelte';
	import MenuItem from '$lib/components/m3/MenuItem.svelte';
	import { persistCollapsed, readInitialCollapsed } from '$lib/components/m3/sidebarCollapse';
	import NavList from '$lib/components/NavList.svelte';
	import type { Profile } from '$lib/types';

	let {
		userEmail,
		profile,
		mobileOpen = $bindable(false),
	}: { userEmail: string; profile: Profile; mobileOpen?: boolean } = $props();

	const STORAGE_KEY = 'nomi:user-sidebar-collapsed';
	let collapsed = $state(false);
	let accountMenuOpen = $state(false);

	onMount(() => {
		collapsed = readInitialCollapsed(STORAGE_KEY);
	});

	function toggleCollapsed() {
		collapsed = !collapsed;
		persistCollapsed(STORAGE_KEY, collapsed);
	}

	const accountLabel = $derived(profile.display_name || userEmail);

	// The desktop "collapse to icon rail" preference shouldn't also shrink the mobile drawer —
	// a temporary overlay with no labels is a worse tap target, so the drawer always renders
	// fully expanded regardless of what's persisted for the desktop rail.
	const effectiveCollapsed = $derived(collapsed && !mobileOpen);

	const NAV = [
		{ href: '/', label: m.nav_home(), icon: IconHome },
		{ href: '/chats', label: m.nav_chats(), icon: IconChatBubble },
		{ href: '/projects', label: m.nav_projects(), icon: IconFolder },
		{ href: '/money', label: m.nav_money(), icon: IconWallet },
		{ href: '/reminders', label: m.nav_reminders(), icon: IconBell },
		{ href: '/memory', label: m.nav_memory(), icon: IconMemory },
	];

	function isActive(href: string): boolean {
		const path = page.url.pathname;
		if (href === '/') return path === '/';
		if (href === '/chats') return path === '/chats' || path.startsWith('/chat/');
		return path === href || path.startsWith(`${href}/`);
	}

	function goToAccountPage(path: string) {
		accountMenuOpen = false;
		mobileOpen = false;
		goto(path);
	}
</script>

{#if mobileOpen}
	<button
		type="button"
		class="fixed inset-0 z-40 md:hidden"
		style="background: color-mix(in srgb, black 40%, transparent); border: none; padding: 0; cursor: default"
		aria-label={m.nav_close_menu()}
		onclick={() => (mobileOpen = false)}
	></button>
{/if}

<aside
	class="fixed inset-y-0 left-0 z-50 flex w-72 flex-col transition-transform duration-200 md:static md:z-auto md:translate-x-0 md:transition-[width] {mobileOpen
		? 'translate-x-0'
		: '-translate-x-full'} {collapsed ? 'md:w-24 md:items-center' : 'md:w-72'}"
	style="background: var(--md-sys-color-surface-container-low)"
>
	<div
		class="flex w-full items-center gap-2 px-4 pt-5 pb-3"
		class:justify-center={effectiveCollapsed}
		class:justify-between={!effectiveCollapsed}
	>
		<a href="/" class="nomi-brand" aria-label={m.nav_nomi_home()} onclick={() => (mobileOpen = false)}>
			<AgentShape size={40} face />
			{#if !effectiveCollapsed}<span class="nomi-brand__word">nomi</span>{/if}
		</a>
		<!-- Wrapped in a plain div rather than passing `hidden`/`md:*` directly to IconButton:
		     IconButton's own scoped `.m3-icon-btn` style sets `display` with higher CSS
		     specificity than a single Tailwind utility class (Svelte appends its own hash class
		     to every scoped selector), so the utility class can't win no matter the breakpoint. -->
		{#if !effectiveCollapsed}
			<div class="hidden md:block">
				<IconButton onclick={toggleCollapsed} aria-label={m.nav_collapse()}>
					<IconChevronLeft />
				</IconButton>
			</div>
		{/if}
		<div class="md:hidden">
			<IconButton onclick={() => (mobileOpen = false)} aria-label={m.nav_close_menu()}>
				<IconClose />
			</IconButton>
		</div>
	</div>

	<form method="POST" action="/?/newChat" use:enhance class={effectiveCollapsed ? 'pt-2' : 'px-4 pt-2'} onsubmit={() => (mobileOpen = false)}>
		<button type="submit" class="nomi-fab" class:nomi-fab--compact={effectiveCollapsed} aria-label={m.nav_new_chat()}>
			<IconPlus size={24} />
			{#if !effectiveCollapsed}<span>{m.nav_new_chat()}</span>{/if}
		</button>
	</form>

	<NavList items={NAV} rail={effectiveCollapsed} {isActive} label={m.nav_main()} onnavigate={() => (mobileOpen = false)} />

	<div class="flex-1"></div>

	{#if effectiveCollapsed}
		<div class="hidden pb-2 md:block">
			<IconButton onclick={toggleCollapsed} aria-label={m.nav_expand()}>
				<IconChevronRight />
			</IconButton>
		</div>
	{/if}

	<div class="w-full px-3 pb-4">
		<Menu bind:open={accountMenuOpen} class="w-full">
			{#snippet trigger({ toggle })}
				<button
					type="button"
					onclick={toggle}
					class="m3-account-trigger w-full"
					class:justify-center={effectiveCollapsed}
					aria-label={m.nav_account_menu()}
				>
					<Avatar name={accountLabel} avatarUrl={profile.avatar_url} size={36} />
					{#if !effectiveCollapsed}
						<span class="md-body-medium truncate" style="color: var(--md-sys-color-on-surface); font-weight: 600">
							{accountLabel}
						</span>
					{/if}
				</button>
			{/snippet}
			<div class="w-full">
				<MenuItem onclick={() => goToAccountPage('/preferences')}>{m.nav_preferences()}</MenuItem>
				<MenuItem onclick={() => goToAccountPage('/profile')}>{m.nav_profile()}</MenuItem>
				<MenuItem onclick={() => goToAccountPage('/account')}>{m.nav_account()}</MenuItem>
				<MenuItem onclick={() => goToAccountPage('/connections')}>{m.nav_connections()}</MenuItem>
				<MenuItem onclick={() => goToAccountPage('/models')}>{m.nav_model()}</MenuItem>
				<MenuItem onclick={() => goToAccountPage('/memory')}>{m.nav_memory()}</MenuItem>
				<form method="POST" action="/logout">
					<MenuItem type="submit">{m.nav_log_out()}</MenuItem>
				</form>
			</div>
		</Menu>
	</div>
</aside>

<style>
	.nomi-brand {
		display: flex;
		align-items: center;
		gap: 10px;
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
	}
	.nomi-brand__word {
		font-family: var(--md-ref-typeface-brand);
		font-size: 2rem;
		line-height: 1;
		font-weight: 800;
		letter-spacing: -0.045em;
	}

	/* The one gradient object in the chrome: starting a conversation is the hero action. */
	.nomi-fab {
		display: flex;
		align-items: center;
		gap: 12px;
		height: 56px;
		width: 100%;
		padding: 0 20px 0 16px;
		border: none;
		border-radius: var(--md-sys-shape-corner-large-increased);
		background: var(--nomi-gradient-glow);
		color: var(--nomi-on-gradient-glow);
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: 1rem;
		font-weight: 650;
		cursor: pointer;
		box-shadow: 0 6px 16px -8px color-mix(in srgb, var(--md-sys-color-primary) 60%, transparent);
		transition:
			border-radius var(--nomi-motion-spatial-fast),
			filter var(--nomi-motion-effects-fast);
	}
	.nomi-fab:hover {
		filter: saturate(1.15) brightness(1.03);
	}
	.nomi-fab:active {
		border-radius: var(--md-sys-shape-corner-medium);
	}
	.nomi-fab--compact {
		width: 56px;
		padding: 0;
		justify-content: center;
		border-radius: var(--md-sys-shape-corner-large);
	}

	.m3-account-trigger {
		display: flex;
		align-items: center;
		gap: 10px;
		border: none;
		background: transparent;
		border-radius: var(--md-sys-shape-corner-full);
		padding: 6px 8px;
		cursor: pointer;
		text-align: left;
	}
	.m3-account-trigger:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
</style>
