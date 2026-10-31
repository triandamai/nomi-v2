<!-- Default, hover and danger: a shortcut or meta sits at the end in mono. -->
<script lang="ts">
	import { GitBranch, Globe, Trash2 } from '@lucide/svelte';
	import MenuRow from '$lib/ds/overlays/MenuRow.svelte';
</script>

<div role="menu" aria-label="Project actions" class="flex w-55 flex-col gap-0.5">
	<MenuRow item={{ label: 'Export repo', icon: GitBranch, shortcut: '⌘E' }} />
	<MenuRow item={{ label: 'Visit live site', icon: Globe, hover: true }} />
	<MenuRow item={{ label: 'Delete project', icon: Trash2, danger: true }} />
</div>
