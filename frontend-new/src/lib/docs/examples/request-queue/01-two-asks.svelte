<!-- Two asks: one change at a time; the next one can go first. -->
<script lang="ts">
	import RequestQueue from '$lib/ds/build/RequestQueue.svelte';
</script>

<div class="w-full max-w-140">
	<RequestQueue
		asks={[
			{ text: 'Make the Pre-order button green', who: { name: 'Mario Prasetyo', initials: 'MP' }, state: 'running', meta: 'Mario · running · 00:21' },
			{ text: 'Add a Makanan section with nasi uduk', who: { name: 'Sari Wulandari', initials: 'SW', peer: 1 }, state: 'next', meta: 'Sari · next' },
		]}
		onfirst={() => {}}
	/>
</div>
