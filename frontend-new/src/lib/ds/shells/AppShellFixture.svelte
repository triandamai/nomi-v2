<script lang="ts">
	import { House } from '@lucide/svelte';
	import AppShell from './AppShell.svelte';
	import { setPanelPlacement } from './panelPlacement';
	import { setMainLayout } from './mainLayout';
	import { setPhoneChrome } from './phoneChrome';

	type Props = { panelUnderTopbar?: boolean; bleed?: boolean; phoneTabs?: boolean; drawer?: boolean };
	let { panelUnderTopbar = false, bleed = false, phoneTabs = false, drawer = false }: Props = $props();
	setPanelPlacement({
		get underTopbar() {
			return panelUnderTopbar;
		},
	});
	setMainLayout({
		get bleed() {
			return bleed;
		},
	});
	setPhoneChrome({
		get phone() {
			return phoneTabs;
		},
		get tabs() {
			return phoneTabs ? { label: 'Main', tabs: [{ href: '/', label: 'Home', icon: House, current: true }] } : null;
		},
		ownTitle: false,
		get drawer() {
			return drawer;
		},
	});
</script>

<AppShell>
	{#snippet sidebar()}<nav aria-label="Main" class="h-full w-62">Sidebar content</nav>{/snippet}
	{#snippet topbar()}<p>Morning, Mario.</p>{/snippet}
	{#snippet panel()}<aside aria-label="Your crew">Crew panel</aside>{/snippet}
	<p>Page content</p>
</AppShell>
