<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { untrack, type Snippet } from 'svelte';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import IconMenu from '$lib/components/icons/IconMenu.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { registerAgentLooks } from '$lib/components/m3/shapes';
	import { getLocale, setLocale } from '$lib/paraglide/runtime';
	import type { LayoutData } from './$types';

	let { data, children }: { data: LayoutData; children: Snippet } = $props();

	let mobileNavOpen = $state(false);

	// Dynamic agents' chosen shapes: registered before any child renders an AgentShape, and again
	// whenever the crew reloads.
	registerAgentLooks(untrack(() => data.crew));
	$effect.pre(() => registerAgentLooks(data.crew));

	// The saved language follows the person to every device: switch (one reload) when this one
	// is still showing another.
	$effect(() => {
		if (data.preferences.language && data.preferences.language !== getLocale()) setLocale(data.preferences.language);
		document.documentElement.lang = getLocale();
	});

	$effect(() => {
		document.documentElement.dataset.theme = data.preferences.theme;
		document.documentElement.dataset.color = data.preferences.accent_color;
	});
</script>

<div class="app-shell flex" style="background: var(--md-sys-color-surface)">
	<Sidebar userEmail={data.userEmail} profile={data.profile} usage={data.usage} bind:mobileOpen={mobileNavOpen} />
	<div class="flex flex-1 flex-col overflow-hidden">
		<header class="flex items-center gap-2 px-2 py-2 md:hidden">
			<IconButton onclick={() => (mobileNavOpen = true)} aria-label={m.nav_open_menu()}>
				<IconMenu />
			</IconButton>
			<a href="/" class="flex items-center gap-2" style="color: var(--md-sys-color-on-surface); text-decoration: none">
				<AgentShape size={28} face />
				<span style="font-family: var(--md-ref-typeface-brand); font-size: 1.5rem; font-weight: 800; letter-spacing: -0.045em">nomi</span>
			</a>
		</header>
		<main class="flex-1 overflow-hidden">
			{@render children()}
		</main>
	</div>
</div>
