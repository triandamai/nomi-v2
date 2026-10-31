<!-- In the chat: a teammate's message in their colour, with when and whom they asked. -->
<script lang="ts">
	import TeammateMessage from '$lib/ds/build/TeammateMessage.svelte';
</script>

<div class="w-full max-w-140">
	<TeammateMessage
		who={{ name: 'Sari Wulandari', initials: 'SW', peer: 1 }}
		time="09:14"
		to="Koda"
		text="Can we add a Makanan section? Nasi uduk and roti bakar, same pickup times."
	/>
</div>
