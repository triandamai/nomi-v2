<!-- At a project's foot: ask Koda for a change; send shows once something is written. -->
<script lang="ts">
	import PhoneAskBar from '$lib/ds/projects/PhoneAskBar.svelte';

	let value = $state('');
</script>

<div class="w-97.5 max-w-full"><PhoneAskBar placeholder="Ask Koda to change something" bind:value onsend={() => {}} /></div>
