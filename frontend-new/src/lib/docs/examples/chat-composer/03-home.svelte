<!-- On Home: roomy, with who it goes to and the keys. -->
<script lang="ts">
	import ChatComposer from '$lib/ds/thread/ChatComposer.svelte';

	let to = $state<'Nomi' | 'Dana'>('Nomi');
</script>

<div class="w-full max-w-208">
	<ChatComposer
		placeholder="Ask Nomi, or hand something to the crew…"
		roomy
		hints
		to={{ agent: to === 'Dana' ? 'dana' : 'nomi', name: to }}
		crew={[
			{ kind: 'crew', agent: 'nomi', name: 'Nomi', role: 'Talks with you' },
			{ kind: 'crew', agent: 'dana', name: 'Dana', role: 'Money and subscriptions' },
		]}
		onto={(name) => (to = name === 'Dana' ? 'Dana' : 'Nomi')}
		onsend={() => {}}
	/>
</div>
