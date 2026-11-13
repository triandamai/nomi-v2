<!-- Folded: open it to show the earlier entries. -->
<script lang="ts">
	import EarlierToggle from '$lib/ds/inbox/EarlierToggle.svelte';

	let expanded = $state(false);
</script>

<div class="flex flex-col gap-2">
	<EarlierToggle label="Earlier" count={5} bind:expanded />
	{#if expanded}<p class="text-caption text-text-3">Five earlier entries would show here.</p>{/if}
</div>
