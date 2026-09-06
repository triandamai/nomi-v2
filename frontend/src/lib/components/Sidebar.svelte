<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { enhance } from '$app/forms';
	import Avatar from '$lib/components/m3/Avatar.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import IconChatBubble from '$lib/components/icons/IconChatBubble.svelte';
	import IconChevronLeft from '$lib/components/icons/IconChevronLeft.svelte';
	import IconChevronRight from '$lib/components/icons/IconChevronRight.svelte';
	import IconFolder from '$lib/components/icons/IconFolder.svelte';
	import IconHistory from '$lib/components/icons/IconHistory.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import Menu from '$lib/components/m3/Menu.svelte';
	import MenuItem from '$lib/components/m3/MenuItem.svelte';
	import { persistCollapsed, readInitialCollapsed } from '$lib/components/m3/sidebarCollapse';
	import type { Profile } from '$lib/types';

	let { userEmail, profile }: { userEmail: string; profile: Profile } = $props();

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

	function goToAccountPage(path: string) {
		accountMenuOpen = false;
		goto(path);
	}
</script>

<aside
	class="flex flex-col transition-[width] duration-200"
	class:w-72={!collapsed}
	class:w-20={collapsed}
	class:items-center={collapsed}
	style="background: var(--md-sys-color-surface-container); border-right: 1px solid var(--md-sys-color-outline-variant)"
>
	<div
		class="flex w-full items-center gap-2 px-4 py-4"
		class:justify-center={collapsed}
		class:justify-between={!collapsed}
		style="border-bottom: 1px solid var(--md-sys-color-outline-variant)"
	>
		{#if !collapsed}
			<span class="md-title-large" style="color: var(--md-sys-color-primary)">Nomi</span>
		{:else}
			<span style="color: var(--md-sys-color-primary)"><IconChatBubble /></span>
		{/if}
		<IconButton onclick={toggleCollapsed} aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}>
			{#if collapsed}
				<IconChevronRight />
			{:else}
				<IconChevronLeft />
			{/if}
		</IconButton>
	</div>

	<form method="POST" action="/?/newChat" use:enhance class={collapsed ? 'pt-3' : 'px-3 pt-3'}>
		{#if collapsed}
			<button type="submit" class="m3-fab" aria-label="New Chat">
				<IconPlus />
			</button>
		{:else}
			<Button type="submit" variant="filled" class="w-full">+ New Chat</Button>
		{/if}
	</form>

	<nav class={collapsed ? 'flex flex-col items-center gap-1 py-3' : 'space-y-1 px-3 py-3'}>
		<a
			href="/chats"
			class="flex items-center gap-2 rounded-full px-3 py-2"
			class:justify-center={collapsed}
			style="color: var(--md-sys-color-on-surface-variant); text-decoration: none;"
		>
			<IconHistory size={20} />
			{#if !collapsed}<span class="md-body-medium">Chats</span>{/if}
		</a>
		<a
			href="/projects"
			class="flex items-center gap-2 rounded-full px-3 py-2"
			class:justify-center={collapsed}
			style="color: var(--md-sys-color-on-surface-variant); text-decoration: none;"
		>
			<IconFolder size={20} />
			{#if !collapsed}<span class="md-body-medium">Projects</span>{/if}
		</a>
	</nav>

	<div class="flex-1"></div>

	<div class="w-full px-2 py-2" style="border-top: 1px solid var(--md-sys-color-outline-variant)">
		<Menu bind:open={accountMenuOpen} class="w-full">
			{#snippet trigger({ toggle })}
				<button
					type="button"
					onclick={toggle}
					class="m3-account-trigger w-full"
					class:justify-center={collapsed}
					aria-label="Account menu"
				>
					<Avatar name={accountLabel} avatarUrl={profile.avatar_url} size={32} />
					{#if !collapsed}
						<span class="md-body-medium truncate" style="color: var(--md-sys-color-on-surface-variant)">
							{accountLabel}
						</span>
					{/if}
				</button>
			{/snippet}
			<div class="w-full">
				<MenuItem onclick={() => goToAccountPage('/preferences')}>Preferences</MenuItem>
				<MenuItem onclick={() => goToAccountPage('/profile')}>Profile</MenuItem>
				<MenuItem onclick={() => goToAccountPage('/account')}>Account settings</MenuItem>
				<form method="POST" action="/logout" use:enhance>
					<MenuItem type="submit">Log out</MenuItem>
				</form>
			</div>
		</Menu>
	</div>
</aside>

<style>
	.m3-fab {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 48px;
		height: 48px;
		border-radius: var(--md-sys-shape-corner-large);
		border: none;
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
		cursor: pointer;
	}
	.m3-fab:hover {
		box-shadow: var(--md-sys-elevation-shadow-level2);
	}

	.m3-account-trigger {
		display: flex;
		align-items: center;
		gap: 8px;
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
