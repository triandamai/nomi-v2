<!-- A radio group: arrow keys move the choice. -->
<script lang="ts">
	import FilterTabs from '$lib/ds/projects/FilterTabs.svelte';
</script>

<FilterTabs
	label="Show projects"
	options={[
		{ value: 'all', label: 'All', count: 4 },
		{ value: 'live', label: 'Live', count: 2 },
		{ value: 'building', label: 'Building', count: 1 },
		{ value: 'attention', label: 'Needs attention', count: 1 },
	]}
/>
