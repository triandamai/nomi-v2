<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Menu } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import IconButton from '../actions/IconButton.svelte';
	import { panelPlacement } from './panelPlacement';
	import { mainLayout } from './mainLayout';

	// Layout shells → App shell (Home, Chats, Projects, Money…): sidebar 248, top bar 52 (24 across,
	// as App / Home Vr4nn and the other app screens draw it), content padded 32–56, an optional
	// panel 300–380. The panel sits beside everything, full height, unless whoever renders the shell
	// puts it under the top bar (setPanelPlacement: Home's crew column, EXxti). Below 1024px the
	// sidebar moves into a drawer; the panel shows only where there's room for it (1280px and up).
	type Props = { sidebar: Snippet; topbar?: Snippet; panel?: Snippet; children: Snippet };
	let { sidebar, topbar, panel, children }: Props = $props();

	const placement = panelPlacement();
	// A screen that scrolls its own parts (a chat) takes the content area edge to edge (mainLayout).
	const layout = mainLayout();
	let drawerOpen = $state(false);

	function drawer(dialog: HTMLDialogElement) {
		dialog.showModal();
		// Following a link inside closes the drawer.
		const leave = (event: MouseEvent) => (event.target as Element).closest('a') && (drawerOpen = false);
		dialog.addEventListener('click', leave);
		return { destroy: () => dialog.removeEventListener('click', leave) };
	}
</script>

{#snippet side()}
	<div class="h-full w-80 shrink-0 overflow-y-auto border-l border-border max-xl:hidden">{@render panel?.()}</div>
{/snippet}

<div class="flex h-dvh w-full overflow-hidden bg-bg text-text">
	<div class="h-full max-lg:hidden">{@render sidebar()}</div>
	<div class="flex min-w-0 flex-1 flex-col">
		<header class="flex h-13 shrink-0 items-center gap-3 border-b border-border px-4 lg:px-6">
			<span class="lg:hidden"><IconButton icon={Menu} label={m.shell_open_nav()} variant="ghost" onclick={() => (drawerOpen = true)} /></span>
			<div class="min-w-0 flex-1">{@render topbar?.()}</div>
		</header>
		<div class="flex min-h-0 flex-1">
			<main class={['min-h-0 min-w-0 flex-1', layout.bleed ? 'overflow-hidden' : 'overflow-y-auto px-4 py-6 lg:px-14 lg:py-8']}>{@render children()}</main>
			{#if panel && placement.underTopbar}{@render side()}{/if}
		</div>
	</div>
	{#if panel && !placement.underTopbar}{@render side()}{/if}
</div>

{#if drawerOpen}
	<dialog
		use:drawer
		aria-label={m.shell_nav_drawer()}
		onclose={() => (drawerOpen = false)}
		class="m-0 h-dvh max-h-none w-auto bg-bg p-0 text-text backdrop:bg-overlay"
	>
		{@render sidebar()}
	</dialog>
{/if}
