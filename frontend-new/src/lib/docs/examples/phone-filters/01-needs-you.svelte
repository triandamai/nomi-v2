<!-- Filters: Needs you first, then the rest. -->
<script lang="ts">
	import PhoneFilters from '$lib/ds/projects/PhoneFilters.svelte';

	let value = $state('attention');
</script>

<div class="w-97.5 max-w-full">
	<PhoneFilters
		label="Show"
		{value}
		onchange={(next) => (value = next)}
		options={[
			{ value: 'attention', label: 'Needs you', count: 1 },
			{ value: 'all', label: 'All', count: 4 },
			{ value: 'live', label: 'Live', count: 2 },
			{ value: 'building', label: 'Building', count: 1 },
		]}
	/>
</div>
