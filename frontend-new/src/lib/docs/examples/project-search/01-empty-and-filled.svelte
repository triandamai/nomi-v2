<!-- Empty and filled: accent once it filters, with a clear. -->
<script lang="ts">
	import ProjectSearch from '$lib/ds/projects/ProjectSearch.svelte';
</script>

<div class="flex flex-wrap gap-6">
	<ProjectSearch />
	<ProjectSearch value="kasir" />
</div>
