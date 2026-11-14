<script lang="ts">
	import ProjectWorkspace from './ProjectWorkspace.svelte';
</script>

<ProjectWorkspace>
	{#snippet projectbar()}<header class="flex h-13 shrink-0 items-center border-b border-border px-4"><p>warung-kopi-menu</p></header>{/snippet}
	{#snippet chat()}<p>Agent chat</p>{/snippet}
	<p>Preview pane</p>
</ProjectWorkspace>
