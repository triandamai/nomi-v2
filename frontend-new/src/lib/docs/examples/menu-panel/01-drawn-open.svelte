<!-- Drawn open: the panel as the sheets show it. In a screen, use Menu. -->
<script lang="ts">
	import { GitBranch, Globe, Pencil, Trash2 } from '@lucide/svelte';
	import MenuPanel from '$lib/ds/overlays/MenuPanel.svelte';
</script>

<MenuPanel
	id="docs-menu-panel"
	aria-label="Project actions"
	title="salah-daily-tracker"
	items={[
		{ label: 'Visit live site', icon: Globe },
		{ label: 'Export repo', icon: GitBranch, shortcut: '⌘E' },
		{ label: 'Rename', icon: Pencil, shortcut: 'F2' },
		'divider',
		{ label: 'Delete project', icon: Trash2, danger: true },
	]}
/>
