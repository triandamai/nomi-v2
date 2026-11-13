<!-- States: open on the page colour, the rest quiet. -->
<script lang="ts">
	import FilterTab from '$lib/ds/projects/FilterTab.svelte';
</script>

<div class="flex gap-6">
	<FilterTab label="All" count={4} active />
	<FilterTab label="Live" count={2} />
</div>
