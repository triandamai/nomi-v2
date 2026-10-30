<script lang="ts">
	import { GitBranch, Globe, Pencil, Trash2 } from '@lucide/svelte';
	import Menu from './Menu.svelte';
	import Button from '../actions/Button.svelte';

	let { onselect }: { onselect: (what: string) => void } = $props();
</script>

<Menu
	title="salah-daily-tracker"
	items={[
		{ label: 'Export repo', icon: GitBranch, shortcut: '⌘E', onselect: () => onselect('export') },
		{ label: 'Visit live site', icon: Globe, onselect: () => onselect('visit') },
		{ label: 'Rename', icon: Pencil, shortcut: 'F2', onselect: () => onselect('rename') },
		'divider',
		{ label: 'Delete project', icon: Trash2, danger: true, onselect: () => onselect('delete') },
	]}
>
	{#snippet trigger(props)}<Button label="Project actions" variant="secondary" {...props} />{/snippet}
</Menu>
<button type="button">Elsewhere</button>
