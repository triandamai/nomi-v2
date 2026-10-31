<!-- With panels: one panel at a time. Left and Right move between tabs. -->
<script lang="ts">
	import Tabs from '$lib/ds/inputs/Tabs.svelte';

	let open = $state('transcript');
</script>

<div class="w-full max-w-120">
	<Tabs
		label="Run details"
		bind:value={open}
		tabs={[
			{ value: 'transcript', label: 'Transcript' },
			{ value: 'events', label: 'Events' },
		]}
	>
		{#snippet panel(value)}
			<p class="pt-4 font-mono text-label text-text-2">
				{value === 'transcript' ? 'Writing src/routes/menu/+page.svelte · 01:12' : 'deploy #14 · main@a3f9c21 · built in 41s · 15:38'}
			</p>
		{/snippet}
	</Tabs>
</div>
