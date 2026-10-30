<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Menu } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import IconButton from '../actions/IconButton.svelte';

	// Twin of frontend-new's AppShell (Home, Chats, Projects, Money…): the navigation drawer, a top
	// app bar, the content, an optional side panel. Below 1024px the drawer becomes a modal one;
	// the panel shows only where there's room for it (1280px and up).
	type Props = { sidebar: Snippet; topbar?: Snippet; panel?: Snippet; children: Snippet };
	let { sidebar, topbar, panel, children }: Props = $props();

	let drawerOpen = $state(false);

	function drawer(dialog: HTMLDialogElement) {
		dialog.showModal();
		// Following a link inside closes the drawer.
		const leave = (event: MouseEvent) => (event.target as Element).closest('a') && (drawerOpen = false);
		dialog.addEventListener('click', leave);
		return { destroy: () => dialog.removeEventListener('click', leave) };
	}
</script>

<div class="shell">
	<div class="side">{@render sidebar()}</div>
	<div class="main">
		<header class="bar">
			<span class="menu"><IconButton icon={Menu} label={m.shell_open_nav()} variant="ghost" onclick={() => (drawerOpen = true)} /></span>
			<div class="title">{@render topbar?.()}</div>
		</header>
		<main class="content">{@render children()}</main>
	</div>
	{#if panel}<div class="panel">{@render panel()}</div>{/if}
</div>
{#if drawerOpen}
	<dialog use:drawer aria-label={m.shell_nav_drawer()} onclose={() => (drawerOpen = false)} class="modal-drawer">
		{@render sidebar()}
	</dialog>
{/if}

<style>
	.shell {
		display: flex;
		height: 100dvh;
		width: 100%;
		overflow: hidden;
		background: var(--nomi-color-ground);
		color: var(--md-sys-color-on-surface);
	}
	.side {
		height: 100%;
	}
	.main {
		display: flex;
		min-width: 0;
		flex: 1;
		flex-direction: column;
	}
	.bar {
		display: flex;
		height: 64px;
		flex-shrink: 0;
		align-items: center;
		gap: 12px;
		padding: 0 16px;
	}
	.title {
		min-width: 0;
		flex: 1;
	}
	.content {
		min-height: 0;
		flex: 1;
		overflow-y: auto;
		padding: 24px 16px;
	}
	.panel {
		height: 100%;
		width: 320px;
		flex-shrink: 0;
		overflow-y: auto;
		background: var(--md-sys-color-surface-container-low);
	}
	.modal-drawer {
		height: 100dvh;
		max-height: none;
		width: auto;
		margin: 0;
		padding: 0;
		border: 0;
		border-radius: 0 var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-large) 0;
		overflow: hidden;
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface);
	}
	.modal-drawer::backdrop {
		background: color-mix(in srgb, var(--md-sys-color-scrim) 32%, transparent);
	}
	@media (max-width: 1023px) {
		.side {
			display: none;
		}
	}
	@media (min-width: 1024px) {
		.menu {
			display: none;
		}
		.bar {
			padding: 0 32px;
		}
		.content {
			padding: 32px 56px;
		}
	}
	@media (max-width: 1279px) {
		.panel {
			display: none;
		}
	}
</style>
