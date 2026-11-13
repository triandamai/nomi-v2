<!-- Drawn open: choose a workspace; the note says what changes for whom. -->
<script lang="ts">
	import ProjectMoveDialog from '$lib/ds/projects/ProjectMoveDialog.svelte';
</script>

<ProjectMoveDialog
	inline
	name="salah-daily-tracker"
	chats={2}
	owner="Trian"
	value="w3"
	workspaces={[
		{ id: 'w1', name: 'Mario', meta: 'personal · only you', tone: 'plain', members: 1 },
		{ id: 'w2', name: "Trian's space", meta: 'current · 3 members', tone: 'nomi', members: 3, current: true },
		{ id: 'w3', name: 'Kopi Kita', meta: 'Admin · 5 members', tone: 'tara', members: 5 },
	]}
	onmove={() => {}}
/>
