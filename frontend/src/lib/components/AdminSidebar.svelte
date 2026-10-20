<script lang="ts">
	import IconChatBubble from '$lib/components/icons/IconChatBubble.svelte';
	import IconRocket from '$lib/components/icons/IconRocket.svelte';
	import IconSettings from '$lib/components/icons/IconSettings.svelte';
	import { VERSION_LABEL } from '$lib/version';
	import { m } from '$lib/paraglide/messages';
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import NavList from '$lib/components/NavList.svelte';
	import IconAgents from '$lib/components/icons/IconAgents.svelte';
	import IconArrowBack from '$lib/components/icons/IconArrowBack.svelte';
	import IconChevronLeft from '$lib/components/icons/IconChevronLeft.svelte';
	import IconChevronRight from '$lib/components/icons/IconChevronRight.svelte';
	import IconChip from '$lib/components/icons/IconChip.svelte';
	import IconClose from '$lib/components/icons/IconClose.svelte';
	import IconDashboard from '$lib/components/icons/IconDashboard.svelte';
	import IconLogout from '$lib/components/icons/IconLogout.svelte';
	import IconMemory from '$lib/components/icons/IconMemory.svelte';
	import IconPalette from '$lib/components/icons/IconPalette.svelte';
	import IconWallet from '$lib/components/icons/IconWallet.svelte';
	import IconBarChart from '$lib/components/icons/IconBarChart.svelte';
	import IconInbox from '$lib/components/icons/IconInbox.svelte';
	import IconLink from '$lib/components/icons/IconLink.svelte';
	import IconPerson from '$lib/components/icons/IconPerson.svelte';
	import IconSparkle from '$lib/components/icons/IconSparkle.svelte';
	import IconTool from '$lib/components/icons/IconTool.svelte';
	import { persistCollapsed, readInitialCollapsed } from '$lib/components/m3/sidebarCollapse';
	import { can, type Level } from '$lib/permissions';

	// The admin console's navigation: the same drawer, rail and phone sheet as the app's sidebar,
	// marked "Admin" so it's never mistaken for the app itself.
	let {
		access,
		mobileOpen = $bindable(false),
		onappearance,
	}: { access: Record<string, Level>; mobileOpen?: boolean; onappearance: () => void } = $props();

	const STORAGE_KEY = 'nomi:admin-sidebar-collapsed';
	let collapsed = $state(false);

	onMount(() => {
		collapsed = readInitialCollapsed(STORAGE_KEY);
	});

	function toggleCollapsed() {
		collapsed = !collapsed;
		persistCollapsed(STORAGE_KEY, collapsed);
	}

	// The phone drawer always shows labels, whatever the desktop rail preference is.
	const rail = $derived(collapsed && !mobileOpen);

	// Each page shows for whoever can at least view its area.
	const NAV = $derived(
		[
			{ href: '/admin', label: m.admin_overview(), icon: IconDashboard, area: 'dashboard' },
			{ href: '/admin/agents', label: m.admin_live_agents(), icon: IconAgents, area: 'dashboard' },
			{ href: '/admin/usage', label: m.admin_usage(), icon: IconBarChart, area: 'dashboard' },
			{ href: '/admin/dynamic-agents', label: m.admin_custom_agents(), icon: IconSparkle, area: 'agents' },
			{ href: '/admin/tools', label: m.admin_tools(), icon: IconTool, area: 'tools' },
			{ href: '/admin/settings/providers', label: m.prov_title(), icon: IconLink, area: 'models' },
			{ href: '/admin/settings/llm', label: m.admin_models(), icon: IconChip, area: 'models' },
			{ href: '/admin/settings/embedding', label: m.admin_embeddings(), icon: IconMemory, area: 'models' },
			{ href: '/admin/integrations', label: m.admin_integrations(), icon: IconRocket, area: 'integrations' },
			{ href: '/admin/support', label: m.admin_support(), icon: IconChatBubble, area: 'support' },
			{ href: '/admin/users', label: m.admin_users(), icon: IconPerson, area: 'user' },
			{ href: '/admin/plans', label: m.admin_plans(), icon: IconWallet, area: 'plans' },
			{ href: '/admin/notifications', label: m.admin_notifications(), icon: IconInbox, area: 'notifications' },
			{ href: '/admin/maintenance', label: m.admin_maintenance(), icon: IconSettings, area: 'maintenance' },
		].filter((item) => can(access, item.area)),
	);

	function isActive(href: string): boolean {
		const path = page.url.pathname;
		return href === '/admin' ? path === '/admin' : path === href || path.startsWith(`${href}/`);
	}
</script>

{#if mobileOpen}
	<button type="button" class="scrim md:hidden" aria-label={m.nav_close_menu()} onclick={() => (mobileOpen = false)}></button>
{/if}

<aside
	class="fixed inset-y-0 left-0 z-50 flex w-72 flex-col transition-transform duration-200 md:static md:z-auto md:translate-x-0 md:transition-[width] {mobileOpen
		? 'translate-x-0'
		: '-translate-x-full'} {collapsed ? 'md:w-24 md:items-center' : 'md:w-72'}"
	style="background: var(--md-sys-color-surface-container-low)"
>
	<div class="flex w-full items-center gap-2 px-4 pt-5 pb-3" class:justify-center={rail} class:justify-between={!rail}>
		<a href="/admin" class="brand" aria-label={m.admin_brand()} onclick={() => (mobileOpen = false)}>
			<AgentShape size={40} face />
			{#if !rail}
				<span class="brand__word">nomi</span>
				<span class="brand__tag">{m.admin_tag()}</span>
			{/if}
		</a>
		{#if !rail}
			<div class="hidden md:block">
				<IconButton onclick={toggleCollapsed} aria-label={m.nav_collapse()}><IconChevronLeft /></IconButton>
			</div>
		{/if}
		<div class="md:hidden">
			<IconButton onclick={() => (mobileOpen = false)} aria-label={m.nav_close_menu()}><IconClose /></IconButton>
		</div>
	</div>

	<!-- Scrolls on its own when the screen is shorter than the list, keeping the brand and the
	     footer actions (appearance, back to app, log out) in place. -->
	<div class="drawer-scroll" data-testid="admin-drawer-scroll">
		<NavList items={NAV} {rail} {isActive} label={m.admin_tag()} onnavigate={() => (mobileOpen = false)} />
	</div>

	{#if rail}
		<div class="hidden flex-col items-center gap-1 pb-4 md:flex">
			<IconButton onclick={toggleCollapsed} aria-label={m.nav_expand()}><IconChevronRight /></IconButton>
			<IconButton onclick={onappearance} aria-label={m.admin_appearance()}><IconPalette /></IconButton>
			<IconButton href="/" aria-label={m.admin_back()}><IconArrowBack /></IconButton>
			<form method="POST" action="/logout?redirect_to=/login">
				<IconButton type="submit" aria-label={m.nav_log_out()}><IconLogout /></IconButton>
			</form>
			<span class="version version--rail" title="Nomi {VERSION_LABEL}">{VERSION_LABEL}</span>
		</div>
	{:else}
		<div class="footer">
			<button
				type="button"
				class="footer__item"
				onclick={() => {
					mobileOpen = false;
					onappearance();
				}}
			>
				<IconPalette size={20} /> {m.admin_appearance()}
			</button>
			<a href="/" class="footer__item" onclick={() => (mobileOpen = false)}>
				<IconArrowBack size={20} /> {m.admin_back()}
			</a>
			<form method="POST" action="/logout?redirect_to=/login">
				<button type="submit" class="footer__item footer__item--quiet"><IconLogout size={20} /> {m.nav_log_out()}</button>
			</form>
			<span class="version">Nomi {VERSION_LABEL}</span>
		</div>
	{/if}
</aside>

<style>
	.drawer-scroll {
		flex: 1;
		min-height: 0;
		width: 100%;
		overflow-y: auto;
		overscroll-behavior: contain;
		scrollbar-width: thin;
	}
	.version {
		display: block;
		padding: 8px 14px 0;
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		letter-spacing: 0.02em;
	}
	.version--rail {
		max-width: 72px;
		padding: 4px 0 0;
		overflow: hidden;
		text-align: center;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.scrim {
		position: fixed;
		inset: 0;
		z-index: 40;
		padding: 0;
		border: none;
		background: color-mix(in srgb, var(--md-sys-color-scrim) 40%, transparent);
		cursor: default;
	}
	.brand {
		display: flex;
		align-items: center;
		gap: 10px;
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
	}
	.brand__word {
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.75rem;
		line-height: 1;
		font-weight: 800;
		letter-spacing: -0.045em;
	}
	.brand__tag {
		padding: 3px 10px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	.footer {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 12px 12px 16px;
	}
	.footer__item {
		display: flex;
		align-items: center;
		gap: 12px;
		width: 100%;
		height: 48px;
		padding: 0 16px;
		border: none;
		border-radius: 24px;
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: 0.9375rem;
		font-weight: 600;
		text-decoration: none;
		cursor: pointer;
		transition: background-color var(--nomi-motion-effects-fast);
	}
	.footer__item:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 6%, transparent);
	}
	.footer__item--quiet {
		color: var(--md-sys-color-outline);
	}
</style>
