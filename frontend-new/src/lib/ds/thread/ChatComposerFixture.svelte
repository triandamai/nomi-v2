<script lang="ts">
	import ChatComposer from './ChatComposer.svelte';

	// For the tests: a chat whose reply starts as soon as a message is sent, as a real one does.
	let { onsend, onstop }: { onsend: (text: string) => void; onstop: () => void } = $props();
	let busy = $state(false);
	let value = $state('Tambah jam buka');
</script>

<ChatComposer
	bind:value
	placeholder="Ask about this project…"
	{busy}
	onsend={(text) => {
		busy = true;
		onsend(text);
	}}
	{onstop}
/>
