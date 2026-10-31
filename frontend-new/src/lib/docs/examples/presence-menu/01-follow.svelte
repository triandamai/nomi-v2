<!-- Follow: who's here, what each is looking at; you come last. -->
<script lang="ts">
	import PresenceMenu from '$lib/ds/build/PresenceMenu.svelte';
</script>

<PresenceMenu
	people={[
		{ name: 'Sari Wulandari', initials: 'SW', peer: 1, where: 'Viewing Code · +page.svelte · active now' },
		{ name: 'Mario Prasetyo', initials: 'MP', where: 'you · Preview', you: true },
	]}
	onfollow={() => {}}
/>
