<script lang="ts">
	import AppShell from './AppShell.svelte';
</script>

<AppShell>
	{#snippet sidebar()}<nav aria-label="Main" class="h-full w-62">Sidebar content</nav>{/snippet}
	{#snippet topbar()}<p>Morning, Mario.</p>{/snippet}
	{#snippet panel()}<aside aria-label="Your crew">Crew panel</aside>{/snippet}
	<p>Page content</p>
</AppShell>
