<!-- The three levels, a step stronger each. -->
<script lang="ts">
	import LevelBadge from '$lib/ds/admin/LevelBadge.svelte';
</script>

<div class="flex items-center gap-2">
	<LevelBadge level="view" label="View" />
	<LevelBadge level="write" label="Write" />
	<LevelBadge level="manage" label="Manage" />
</div>
