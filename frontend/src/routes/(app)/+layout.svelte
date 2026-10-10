<script lang="ts">
	import { enablePageTransitions } from '$lib/pageTransitions';
	import { m } from '$lib/paraglide/messages';
	import { untrack, type Snippet } from 'svelte';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import IconMenu from '$lib/components/icons/IconMenu.svelte';
	import Sidebar from '$lib/components/Sidebar.svelte';
	import { registerAgentLooks } from '$lib/components/m3/shapes';
	import IconInbox from '$lib/components/icons/IconInbox.svelte';
	import { badgeLabel, inbox, pollUnread } from '$lib/notifications.svelte';
	import { getLocale, setLocale } from '$lib/paraglide/runtime';
	import type { LayoutData } from './$types';

	let { data, children }: { data: LayoutData; children: Snippet } = $props();

	enablePageTransitions();

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

	// The bell's count: from the server on every navigation, then polled.
	$effect(() => {
		inbox.unread = data.unread;
	});
	$effect(() => pollUnread());

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
			<a href="/notifications" class="bell" aria-label={inbox.unread > 0 ? m.notif_bell_unread({ count: inbox.unread }) : m.notif_title()}>
				<IconInbox size={24} />
				{#if inbox.unread > 0}<span class="bell__badge" aria-hidden="true">{badgeLabel(inbox.unread)}</span>{/if}
			</a>
		</header>
		<main class="flex-1 overflow-hidden" style="view-transition-name: page">
			{@render children()}
		</main>
	</div>
</div>

<style>
	.bell {
		position: relative;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 48px;
		height: 48px;
		margin-left: auto;
		border-radius: var(--md-sys-shape-corner-full);
		color: var(--md-sys-color-on-surface-variant);
	}
	.bell:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.bell__badge {
		position: absolute;
		top: 6px;
		right: 4px;
		min-width: 18px;
		height: 18px;
		padding: 0 5px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-error);
		color: var(--md-sys-color-on-error);
		font-size: 0.6875rem;
		font-weight: 700;
		line-height: 18px;
		text-align: center;
	}
</style>
