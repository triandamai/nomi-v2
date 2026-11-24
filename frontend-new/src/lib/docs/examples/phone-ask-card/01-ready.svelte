<!-- Ask card: something typed, so Send is filled. -->
<script lang="ts">
	import PhoneAskCard from '$lib/ds/thread/PhoneAskCard.svelte';

	let value = $state('Remind me to check pre-orders at 4pm');
</script>

<div class="w-97.5 max-w-full">
	<PhoneAskCard bind:value placeholder="Ask Nomi, or hand something to the crew…" to={{ agent: 'nomi', name: 'Nomi' }} model="deepseek-flash" onsend={() => (value = '')} />
</div>
