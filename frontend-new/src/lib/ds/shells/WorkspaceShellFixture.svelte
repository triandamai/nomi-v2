<script lang="ts">
	import WorkspaceShell from './WorkspaceShell.svelte';
</script>

<WorkspaceShell>
	{#snippet projectbar()}<p>warung-kopi-menu</p>{/snippet}
	{#snippet chat()}<p>Agent chat</p>{/snippet}
	<p>Preview pane</p>
</WorkspaceShell>
