<script lang="ts">
	import { Table2 } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Build — Workspace tools (U16wU) → Screen / Database (xYgNi) → Table: one table of the
	// project's database, [6,10] and 8 apart: its icon, its name in mono and its rows; the one shown
	// filled with surface-2, the others outlined.
	type Props = { name: string; rows: string; selected?: boolean; onclick?: () => void };
	let { name, rows, selected = false, onclick }: Props = $props();
</script>

<button
	type="button"
	aria-pressed={selected}
	{onclick}
	class={['inline-flex h-7.25 cursor-pointer items-center gap-2 rounded-sm px-2.5 outline-offset-1', selected ? 'bg-surface-2 text-text' : 'text-text-2 outline -outline-offset-1 outline-border hover:text-text']}
>
	<span class={['inline-flex', selected ? 'text-text' : 'text-text-3']}><Icon icon={Table2} size={14} /></span>
	<span class="font-mono text-ui">{name}</span>
	<span class="font-mono text-label text-text-3">{rows}</span>
</button>
