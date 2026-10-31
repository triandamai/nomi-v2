<!-- With counts: views of one list. Arrow keys move the choice. -->
<script lang="ts">
	import Segmented from '$lib/ds/inputs/Segmented.svelte';

	let view = $state('all');
</script>

<Segmented
	label="Projects"
	bind:value={view}
	options={[
		{ value: 'all', label: 'All', count: 4 },
		{ value: 'live', label: 'Live', count: 2 },
		{ value: 'building', label: 'Building', count: 1 },
	]}
/>
