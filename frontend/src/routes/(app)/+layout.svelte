<script lang="ts">
	import type { Snippet } from 'svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import IconMenu from '$lib/components/icons/IconMenu.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import type { LayoutData } from './$types';

	let { data, children }: { data: LayoutData; children: Snippet } = $props();

	let mobileNavOpen = $state(false);

	$effect(() => {
		document.documentElement.dataset.theme = data.preferences.theme;
		document.documentElement.dataset.color = data.preferences.accent_color;
	});
</script>

<div class="flex h-screen" style="background: var(--md-sys-color-surface)">
	<Sidebar userEmail={data.userEmail} profile={data.profile} bind:mobileOpen={mobileNavOpen} />
	<div class="flex flex-1 flex-col overflow-hidden">
		<header
			class="flex items-center gap-2 px-3 py-2 md:hidden"
			style="border-bottom: 1px solid var(--md-sys-color-outline-variant)"
		>
			<IconButton onclick={() => (mobileNavOpen = true)} aria-label="Open menu">
				<IconMenu />
			</IconButton>
			<span class="md-title-medium" style="color: var(--md-sys-color-primary)">Nomi</span>
		</header>
		<main class="flex-1 overflow-hidden">
			{@render children()}
		</main>
	</div>
</div>
