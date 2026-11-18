<!-- Workspaces: the one projects come from, and what stays personal. -->
<script lang="ts">
	import PhoneWorkspaceList from '$lib/ds/projects/PhoneWorkspaceList.svelte';

	let chosen = $state('trian');
</script>

<div class="w-97.5 max-w-full rounded-t-sheet bg-surface pb-4 outline -outline-offset-1 outline-border-strong">
	<PhoneWorkspaceList
		workspaces={[
			{ id: 'mario', name: 'Mario', meta: 'only you · 1 project' },
			{ id: 'trian', name: 'Trian’s space', meta: 'Admin · 3 members · 4 projects' },
		]}
		{chosen}
		onchoose={(id) => (chosen = id)}
	/>
</div>
