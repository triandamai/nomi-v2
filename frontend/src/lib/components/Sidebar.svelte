<script lang="ts">
	import { onMount } from 'svelte';
	import { enhance } from '$app/forms';
	import SessionListItem from './SessionListItem.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Icon from '$lib/components/m3/Icon.svelte';
	import { persistCollapsed, readInitialCollapsed } from '$lib/components/m3/sidebarCollapse';
	import type { SessionSummary } from '$lib/types';

	let { sessions, userEmail }: { sessions: SessionSummary[]; userEmail: string } = $props();

	const STORAGE_KEY = 'nomi:user-sidebar-collapsed';
	let collapsed = $state(false);

	onMount(() => {
		collapsed = readInitialCollapsed(STORAGE_KEY);
	});

	function toggleCollapsed() {
		collapsed = !collapsed;
		persistCollapsed(STORAGE_KEY, collapsed);
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
			<span style="color: var(--md-sys-color-primary)"><Icon name="chat-bubble" /></span>
		{/if}
		<button
			type="button"
			class="m3-icon-button"
			onclick={toggleCollapsed}
			aria-label={collapsed ? 'Expand sidebar' : 'Collapse sidebar'}
		>
			<Icon name={collapsed ? 'chevron-right' : 'chevron-left'} />
		</button>
	</div>

	<form method="POST" action="/?/newChat" use:enhance class={collapsed ? 'pt-3' : 'px-3 pt-3'}>
		{#if collapsed}
			<button type="submit" class="m3-fab" aria-label="New Chat">
				<Icon name="plus" />
			</button>
		{:else}
			<Button type="submit" variant="filled" class="w-full">+ New Chat</Button>
		{/if}
	</form>

	{#if !collapsed}
		<nav class="flex-1 space-y-1 overflow-y-auto px-3 py-3">
			{#each sessions as session (session.id)}
				<SessionListItem {session} />
			{/each}
		</nav>
	{:else}
		<div class="flex-1"></div>
	{/if}

	<form
		method="POST"
		action="/logout"
		class="flex w-full items-center px-4 py-3"
		class:justify-center={collapsed}
		class:justify-between={!collapsed}
		style="border-top: 1px solid var(--md-sys-color-outline-variant)"
	>
		{#if !collapsed}
			<span class="md-body-medium truncate" style="color: var(--md-sys-color-on-surface-variant)">{userEmail}</span>
			<button
				type="submit"
				class="md-label-large"
				style="color: var(--md-sys-color-outline); background: none; border: none; cursor: pointer"
			>
				Log out
			</button>
		{:else}
			<button type="submit" class="m3-icon-button" style="color: var(--md-sys-color-outline)" aria-label="Log out">
				<Icon name="logout" />
			</button>
		{/if}
	</form>
</aside>

<style>
	.m3-icon-button {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		border-radius: var(--md-sys-shape-corner-full);
		border: none;
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		cursor: pointer;
	}
	.m3-icon-button:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}

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
</style>
