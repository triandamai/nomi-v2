<!-- From the chip: press it, then pick where projects come from. The chosen one is checked. -->
<script lang="ts">
	import WorkspaceSwitcher, { type Workspace } from '$lib/ds/chrome/WorkspaceSwitcher.svelte';

	let chosen = $state('o2');
	const all = [
		{ id: 'o1', name: 'Mario', tone: 'plain', meta: 'Owner · only you · 1 project' },
		{ id: 'o2', name: "Trian's space", tone: 'teal', meta: 'Admin · 3 members · 4 projects' },
		{ id: 'o3', name: 'Kopi Kita', tone: 'amber', meta: 'Member · 2 members · 2 projects' },
	] as const;
	const workspaces: Workspace[] = $derived(all.map((workspace) => ({ ...workspace, chosen: workspace.id === chosen })));
</script>

<WorkspaceSwitcher
	current={all.find((workspace) => workspace.id === chosen)?.name ?? ''}
	{workspaces}
	settingsHref="#workspace-settings"
	onchoose={(id) => (chosen = id)}
	oncreate={() => {}}
/>
