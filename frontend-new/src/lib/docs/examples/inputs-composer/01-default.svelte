<!-- Composer: type and press Enter to send; send stays off until there is text. -->
<script lang="ts">
	import Composer from '$lib/ds/inputs/Composer.svelte';

	let value = $state('');
</script>

<div class="w-full max-w-180">
	<Composer
		bind:value
		placeholder="Ask Nomi, or hand something to the crew…"
		agent={{ id: 'nomi', name: 'Nomi' }}
		model="deepseek-flash"
		onsend={() => {}}
	/>
</div>
