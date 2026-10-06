<script lang="ts">
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
	import IconPerson from '$lib/components/icons/IconPerson.svelte';
	import IconSparkle from '$lib/components/icons/IconSparkle.svelte';
	import { persistCollapsed, readInitialCollapsed } from '$lib/components/m3/sidebarCollapse';

	// The admin console's navigation: the same drawer, rail and phone sheet as the app's sidebar,
	// marked "Admin" so it's never mistaken for the app itself.
	let {
		canManageSystemConfig,
		canViewUsers,
		mobileOpen = $bindable(false),
		onappearance,
	}: { canManageSystemConfig: boolean; canViewUsers: boolean; mobileOpen?: boolean; onappearance: () => void } = $props();

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

	const NAV = $derived([
		...(canManageSystemConfig
			? [
					{ href: '/admin', label: m.admin_overview(), icon: IconDashboard },
					{ href: '/admin/agents', label: m.admin_live_agents(), icon: IconAgents },
					{ href: '/admin/dynamic-agents', label: m.admin_custom_agents(), icon: IconSparkle },
					{ href: '/admin/settings/llm', label: m.admin_models(), icon: IconChip },
					{ href: '/admin/settings/embedding', label: m.admin_embeddings(), icon: IconMemory },
				]
			: []),
		...(canViewUsers ? [{ href: '/admin/users', label: m.admin_users(), icon: IconPerson }] : []),
	]);

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

	<NavList items={NAV} {rail} {isActive} label={m.admin_tag()} onnavigate={() => (mobileOpen = false)} />

	<div class="flex-1"></div>

	{#if rail}
		<div class="hidden flex-col items-center gap-1 pb-4 md:flex">
			<IconButton onclick={toggleCollapsed} aria-label={m.nav_expand()}><IconChevronRight /></IconButton>
			<IconButton onclick={onappearance} aria-label={m.admin_appearance()}><IconPalette /></IconButton>
			<IconButton href="/" aria-label={m.admin_back()}><IconArrowBack /></IconButton>
			<form method="POST" action="/logout?redirect_to=/login">
				<IconButton type="submit" aria-label={m.nav_log_out()}><IconLogout /></IconButton>
			</form>
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
		</div>
	{/if}
</aside>

<style>
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
		font-size: 2rem;
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
		padding: 12px 12px 20px;
	}
	.footer__item {
		display: flex;
		align-items: center;
		gap: 14px;
		width: 100%;
		height: 48px;
		padding: 0 20px;
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
