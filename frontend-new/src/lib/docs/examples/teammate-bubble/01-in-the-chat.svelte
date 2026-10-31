<!-- In the chat: their colour, their name, and whom they asked. -->
<script lang="ts">
	import TeammateBubble from '$lib/ds/mobile/TeammateBubble.svelte';
</script>

<div class="w-89.5 max-w-full">
	<TeammateBubble
		who={{ name: 'Sari', initials: 'SW', peer: 1 }}
		time="09:14"
		to="Koda"
		text="Can we add a Makanan section? Nasi uduk and roti bakar, same pickup times."
	/>
</div>
