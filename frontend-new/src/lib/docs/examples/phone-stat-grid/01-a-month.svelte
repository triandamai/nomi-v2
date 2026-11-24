<!-- A month of usage, two to a row. -->
<script lang="ts">
	import PhoneStatGrid from '$lib/ds/settings/PhoneStatGrid.svelte';
</script>

<div class="w-97.5 max-w-full px-4">
	<PhoneStatGrid
		tiles={[
			{ label: 'Spend at list price', value: '$0.42', note: 'not what you pay' },
			{ label: 'Requests', value: '86', note: 'across 3 models' },
			{ label: 'Tokens in', value: '241K', note: 'prompts + memory' },
			{ label: 'Tokens out', value: '71K', note: 'replies' },
		]}
	/>
</div>
