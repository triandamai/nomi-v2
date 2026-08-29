<script lang="ts">
	import { onMount } from 'svelte';
	import type { Snippet } from 'svelte';
	import Icon from '$lib/components/m3/Icon.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import { persistCollapsed, readInitialCollapsed } from '$lib/components/m3/sidebarCollapse';
	import type { LayoutData } from './$types';

	let { children }: { data: LayoutData; children: Snippet } = $props();

	const STORAGE_KEY = 'nomi:admin-sidebar-collapsed';
	let collapsed = $state(false);

	onMount(() => {
		collapsed = readInitialCollapsed(STORAGE_KEY);
	});

	function toggleCollapsed() {
		collapsed = !collapsed;
		persistCollapsed(STORAGE_KEY, collapsed);
	}
</script>

<div class="flex h-screen" style="background: var(--md-sys-color-surface)">
	<aside
		class="flex flex-col p-4 transition-[width] duration-200"
		class:w-56={!collapsed}
		class:w-20={collapsed}
		class:items-center={collapsed}
		style="background: var(--md-sys-color-surface-container); border-right: 1px solid var(--md-sys-color-outline-variant)"
	>
		<div class="mb-4 flex w-full items-center" class:justify-center={collapsed} class:justify-between={!collapsed}>
			{#if !collapsed}
				<h2 class="md-title-large" style="color: var(--md-sys-color-on-surface)">Admin</h2>
			{/if}
			<IconButton onclick={toggleCollapsed} aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}>
				<Icon name={collapsed ? 'chevron-right' : 'chevron-left'} />
			</IconButton>
		</div>

		<nav class="flex flex-col gap-1" class:items-center={collapsed}>
			{#if collapsed}
				<IconButton href="/admin" aria-label="Dashboard">
					<Icon name="dashboard" />
				</IconButton>
				<IconButton href="/admin/settings/llm" aria-label="LLM Settings">
					<Icon name="settings" />
				</IconButton>
				<IconButton href="/admin/agents" aria-label="Agents">
					<Icon name="agents" />
				</IconButton>
			{:else}
				<a href="/admin" class="m3-nav-link">Dashboard</a>
				<a href="/admin/settings/llm" class="m3-nav-link">LLM Settings</a>
				<a href="/admin/agents" class="m3-nav-link">Agents</a>
			{/if}
		</nav>

		<form method="POST" action="/logout?redirect_to=/admin/login" class="mt-auto">
			{#if collapsed}
				<IconButton type="submit" style="color: var(--md-sys-color-outline)" aria-label="Log out">
					<Icon name="logout" />
				</IconButton>
			{:else}
				<button type="submit" class="m3-nav-link m3-nav-link--muted w-full text-left">Log out</button>
			{/if}
		</form>
	</aside>
	<main class="flex-1 overflow-y-auto p-8">
		{@render children()}
	</main>
</div>

<style>
	.m3-nav-link {
		display: block;
		padding: 8px 12px;
		border-radius: var(--md-sys-shape-corner-full);
		font-family: var(--md-sys-typescale-label-large-font);
		font-weight: var(--md-sys-typescale-label-large-weight);
		font-size: var(--md-sys-typescale-label-large-size);
		letter-spacing: var(--md-sys-typescale-label-large-tracking);
		color: var(--md-sys-color-on-surface-variant);
		text-decoration: none;
		border: none;
		background: transparent;
		cursor: pointer;
	}
	.m3-nav-link:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.m3-nav-link--muted {
		color: var(--md-sys-color-outline);
	}
</style>
