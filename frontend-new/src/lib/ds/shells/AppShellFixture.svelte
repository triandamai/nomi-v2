<script lang="ts">
	import AppShell from './AppShell.svelte';
	import { setPanelPlacement } from './panelPlacement';

	let { panelUnderTopbar = false }: { panelUnderTopbar?: boolean } = $props();
	setPanelPlacement({
		get underTopbar() {
			return panelUnderTopbar;
		},
	});
</script>

<AppShell>
	{#snippet sidebar()}<nav aria-label="Main" class="h-full w-62">Sidebar content</nav>{/snippet}
	{#snippet topbar()}<p>Morning, Mario.</p>{/snippet}
	{#snippet panel()}<aside aria-label="Your crew">Crew panel</aside>{/snippet}
	<p>Page content</p>
</AppShell>
