<script lang="ts">
	import Tabs from './Tabs.svelte';

	let value = $state('transcript');
</script>

<Tabs
	label="Run details"
	tabs={[
		{ value: 'transcript', label: 'Transcript' },
		{ value: 'events', label: 'Events' },
	]}
	bind:value
>
	{#snippet panel(open)}<p>{open === 'transcript' ? 'Transcript panel' : 'Events panel'}</p>{/snippet}
</Tabs>
