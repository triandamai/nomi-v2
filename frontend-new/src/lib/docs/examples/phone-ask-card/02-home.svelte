<!-- On Home: room to write, and a + that opens a new chat. -->
<script lang="ts">
	import PhoneAskCard from '$lib/ds/thread/PhoneAskCard.svelte';

	let value = $state('');
</script>

<div class="w-97.5 max-w-full">
	<PhoneAskCard bind:value placeholder="Ask Nomi, or hand something to the crew…" to={{ agent: 'nomi', name: 'Nomi' }} model="Gemini 3.6 Flash" roomy start={{ label: 'New chat', href: '/chat/new' }} onsend={() => (value = '')} />
</div>
