<!-- Workspace and people: the crumb opens the workspace switcher; Invite asks who to add. -->
<script lang="ts">
	import ProjectTopBar from '$lib/ds/projects/ProjectTopBar.svelte';

	let chosen = $state('w2');
	const workspaces = $derived(
		[
			{ id: 'w1', name: 'Mario', tone: 'plain' as const, meta: 'only you · 1 project', tag: 'Personal' },
			{ id: 'w2', name: "Trian's space", tone: 'teal' as const, meta: 'Admin · 3 members · 4 projects' },
		].map((workspace) => ({ ...workspace, chosen: workspace.id === chosen })),
	);
</script>

<ProjectTopBar
	{workspaces}
	onchoose={(id) => (chosen = id)}
	oncreate={() => {}}
	members={[
		{ initials: 'T', name: 'Trian', tone: 'teal' },
		{ initials: 'MP', name: 'Mario Prasetyo', tone: 'plain' },
		{ initials: 'SR', name: 'Siti Rahma', tone: 'orange' },
	]}
	oninvite={() => {}}
/>
