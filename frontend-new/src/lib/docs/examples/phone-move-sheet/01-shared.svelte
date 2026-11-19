<!-- A shared project picked: the note says who will see the chat. -->
<script lang="ts">
	import PhoneMoveSheet from '$lib/ds/thread/PhoneMoveSheet.svelte';

	let open = $state(false);
	const projects = [
		{ id: 'p1', name: 'budget-keluarga', workspace: 'Keluarga workspace', shared: 'Keluarga' },
		{ id: 'p2', name: 'salah-daily-tracker', workspace: 'Personal workspace' },
		{ id: 'p3', name: 'kos-mawar-tenants', workspace: 'Personal workspace' },
	];
</script>

<button type="button" class="rounded-sm border border-border-strong px-3 py-1.5 text-ui text-text" onclick={() => (open = true)}>Move to a project</button>
<PhoneMoveSheet bind:open {projects} onmove={() => {}} />
