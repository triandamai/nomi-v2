<!-- In the top bar: who's in the project, the one online marked in mint. -->
<script lang="ts">
	import Presence from '$lib/ds/build/Presence.svelte';
</script>

<Presence
	people={[
		{ name: 'Mario Prasetyo', initials: 'MP' },
		{ name: 'Sari Wulandari', initials: 'SW', peer: 1, online: true },
	]}
/>
