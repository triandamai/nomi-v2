<!-- Move: to another workspace, the current one quiet. -->
<script lang="ts">
	import ProjectMoveSheet from '$lib/ds/projects/ProjectMoveSheet.svelte';
	import Button from '$lib/ds/actions/Button.svelte';

	let open = $state(false);
	let value = $state<string>();
</script>

<Button variant="secondary" label="Move salah-tracker" onclick={() => (open = true)} />
<ProjectMoveSheet
	bind:open
	bind:value
	name="salah-tracker"
	chats={2}
	workspaces={[
		{ id: 'o1', name: 'Mario', meta: 'personal · only you', tone: 'plain', members: 1 },
		{ id: 'o2', name: "Trian's space", meta: '3 members', tone: 'plain', members: 3, current: true },
		{ id: 'o3', name: 'Kopi Kita', meta: 'Admin · 5 members', tone: 'plain', members: 5, owner: 'Andi' },
	]}
	onmove={() => {}}
/>
