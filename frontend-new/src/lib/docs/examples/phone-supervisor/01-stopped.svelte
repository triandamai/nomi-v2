<!-- Stopped: each task's real state. -->
<script lang="ts">
	import PhoneSupervisor from '$lib/ds/thread/PhoneSupervisor.svelte';
</script>

<div class="w-89.5">
	<PhoneSupervisor
		tasks={[
			{ agent: 'nomi', title: 'Find Whoosh trains', state: 'done' },
			{ agent: 'nomi', title: 'Shortlist guesthouses', detail: '2 of 3', state: 'stopped' },
			{ agent: 'tara', title: 'Share with Sari', state: 'waiting' },
		]}
		onresume={() => {}}
	/>
</div>
