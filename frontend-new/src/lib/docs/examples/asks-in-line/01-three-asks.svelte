<!-- Sari's ask running, Mario's next, and Sari's second one waiting behind his: she can go first. -->
<script lang="ts">
	import AsksInLine from '$lib/ds/projects/AsksInLine.svelte';

	const sari = { name: 'Sari Wulandari', initials: 'SW', peer: 1 as const };
	const mario = { name: 'Mario Prasetyo', initials: 'MP' };
</script>

<div class="w-full max-w-120">
	<AsksInLine
		asks={[
			{ text: 'Rename the shop to Warung Kopi Bu Siti', who: sari, state: 'running', meta: 'Sari Wulandari · running' },
			{ text: 'Make the Pre-order button green', who: mario, state: 'next', meta: 'Mario Prasetyo · next' },
			{ text: 'Add a Makanan section with nasi uduk', who: sari, state: 'queued', meta: 'Sari Wulandari · waiting' },
		]}
		first={2}
		onfirst={() => {}}
	/>
</div>
