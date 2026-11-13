<!-- Open, as the pen draws it: a tag after a name, the chosen one filled and checked. -->
<script lang="ts">
	import WorkspaceSwitcher from '$lib/ds/chrome/WorkspaceSwitcher.svelte';
</script>

<WorkspaceSwitcher
	inline
	current="Trian's space"
	workspaces={[
		{ id: 'o1', name: 'Mario', tone: 'plain', meta: 'only you · 1 project', tag: 'Personal', chosen: false },
		{ id: 'o2', name: "Trian's space", tone: 'teal', meta: 'Admin · 3 members · 4 projects', chosen: true },
		{ id: 'o3', name: 'Kopi Kita', tone: 'amber', meta: 'Member · 2 members · 2 projects', chosen: false },
	]}
	settingsHref="#workspace-settings"
	onchoose={() => {}}
	oncreate={() => {}}
/>
