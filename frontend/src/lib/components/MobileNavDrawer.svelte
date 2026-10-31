<script lang="ts">
	import { tick } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { page } from '$app/state';
	import { afterNavigate } from '$app/navigation';
	import NavDrawer from '$lib/components/NavDrawer.svelte';
	import SettingsDrawer from '$lib/components/SettingsDrawer.svelte';
	import { activeSetting } from '$lib/settingsNav';
	import type { RecentsSettings } from '$lib/drawerRecents';
	import type { Profile, UsageBrief } from '$lib/types';

	// The phone's navigation drawer over the page: the same design-system drawer as the desktop
	// sidebar (or, on an account page, the settings one). Unlike the desktop drawer it scrolls as
	// one: brand, New chat, the sections, recents and the person all move together, so a short
	// screen reaches every part of it.
	let {
		open = $bindable(false),
		userEmail,
		profile,
		usage = null,
		savedPins = null,
		recents,
	}: { open?: boolean; userEmail: string; profile: Profile; usage?: UsageBrief | null; savedPins?: string[] | null; recents: RecentsSettings } = $props();

	let panel: HTMLElement | undefined = $state();

	// Going somewhere closes it.
	afterNavigate(() => (open = false));

	$effect(() => {
		if (!open) return;
		tick().then(() => panel?.focus());
		const onkey = (event: KeyboardEvent) => {
			if (event.key === 'Escape') open = false;
		};
		window.addEventListener('keydown', onkey);
		return () => window.removeEventListener('keydown', onkey);
	});
</script>

{#if open}
	<button type="button" class="scrim" aria-label={m.nav_close_menu()} onclick={() => (open = false)}></button>
	<div bind:this={panel} class="panel" role="dialog" aria-modal="true" aria-label={m.nav_main()} tabindex="-1">
		{#if activeSetting(page.url.pathname)}
			<SettingsDrawer {userEmail} {profile} />
		{:else}
			<NavDrawer {userEmail} {profile} {usage} {savedPins} {recents} oncollapse={() => (open = false)} />
		{/if}
	</div>
{/if}

<style>
	.scrim {
		position: fixed;
		inset: 0;
		z-index: 40;
		padding: 0;
		border: none;
		background: color-mix(in srgb, black 40%, transparent);
		cursor: default;
	}
	/* The whole drawer scrolls, top to bottom: the Sidebar twin's own layout keeps its recents
	   scrolling inside a fixed column, which on a phone leaves most of the drawer out of reach. */
	.panel {
		position: fixed;
		inset: 0 auto 0 0;
		z-index: 50;
		width: min(320px, 88vw);
		height: 100dvh;
		overflow-y: auto;
		overscroll-behavior: contain;
		background: var(--md-sys-color-surface-container-low);
		outline: none;
	}
	.panel :global(.drawer) {
		width: 100%;
		height: auto;
		min-height: 100%;
		padding-bottom: max(12px, env(safe-area-inset-bottom));
	}
	.panel :global(.recents) {
		flex: 1 0 auto;
		overflow: visible;
	}
</style>
