<!-- The crew: chats, and Koda's projects. -->
<script lang="ts">
	import CrewUsage from '$lib/ds/profile/CrewUsage.svelte';
</script>

<div class="w-full max-w-56">
	<CrewUsage
		crew={[
			{ agent: 'nomi', name: 'Nomi', count: '142', share: 1 },
			{ agent: 'dana', name: 'Dana', count: '31', share: 0.22 },
			{ agent: 'koda', name: 'Koda', count: '2 projects', share: 0.01 },
		]}
		empty="No chats with the crew yet."
	/>
</div>
