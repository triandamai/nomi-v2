<!-- Project actions: press the button, then use the arrow keys. The destructive item comes last. -->
<script lang="ts">
	import { GitBranch, Globe, Pencil, Trash2 } from '@lucide/svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import Menu from '$lib/ds/overlays/Menu.svelte';
</script>

<Menu
	title="salah-daily-tracker"
	items={[
		{ label: 'Visit live site', icon: Globe, onselect: () => {} },
		{ label: 'Export repo', icon: GitBranch, shortcut: '⌘E', onselect: () => {} },
		{ label: 'Rename', icon: Pencil, shortcut: 'F2', onselect: () => {} },
		'divider',
		{ label: 'Delete project', icon: Trash2, danger: true, onselect: () => {} },
	]}
>
	{#snippet trigger(props)}<Button label="Project actions" variant="secondary" {...props} />{/snippet}
</Menu>
