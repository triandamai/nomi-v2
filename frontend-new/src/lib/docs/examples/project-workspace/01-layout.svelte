<!-- The bar, the chat and the preview pane, at the size of the box it's in. -->
<script lang="ts">
	import ProjectWorkspace from '$lib/ds/projects/ProjectWorkspace.svelte';
</script>

<div class="h-100 w-full overflow-hidden rounded-md outline -outline-offset-1 outline-border">
	<ProjectWorkspace>
		{#snippet projectbar()}<header class="flex h-13 shrink-0 items-center border-b border-border px-4 text-ui font-medium">warung-kopi-menu</header>{/snippet}
		{#snippet chat()}<p class="p-5 text-ui text-text-2">The project’s chat</p>{/snippet}
		<p class="p-5 text-ui text-text-2">The preview pane</p>
	</ProjectWorkspace>
</div>
