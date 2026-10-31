<!-- Together on a phone: two asks in line, and a thumb-sized way to let the next one go first. -->
<script lang="ts">
	import AskQueue from '$lib/ds/mobile/AskQueue.svelte';
</script>

<div class="w-89.5 max-w-full">
	<AskQueue
		asks={[
			{ text: 'Make the Pre-order button green', who: { name: 'Mario', initials: 'MP' }, state: 'running', meta: 'Mario · running · 00:21' },
			{ text: 'Add a Makanan section with nasi uduk', who: { name: 'Sari', initials: 'SW', peer: 1 }, state: 'next', meta: 'Sari · next' },
		]}
		onfirst={() => {}}
	/>
</div>
