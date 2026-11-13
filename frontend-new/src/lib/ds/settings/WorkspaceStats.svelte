<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// App / Settings / Workspace (wZm9B) → In This Workspace: the workspace in numbers, a 64px strip
	// outlined in the quiet border, the stats side by side with a stroke between them, each [14,16]
	// and 10 apart: its icon, the number (15/500 mono) over what it counts.
	type Props = { label: string; stats: { icon: Component<LucideProps>; value: number; label: string }[] };
	let { label, stats }: Props = $props();
</script>

<ul aria-label={label} class="grid auto-cols-fr grid-flow-row rounded-md outline -outline-offset-1 outline-border sm:grid-flow-col">
	{#each stats as stat, index (stat.label)}
		<li class={['flex h-16 items-center gap-2.5 px-4', index > 0 && 'border-t border-border sm:border-t-0 sm:border-l']}>
			<span class="inline-flex text-text-2"><Icon icon={stat.icon} /></span>
			<span class="flex flex-col gap-px">
				<span class="font-mono text-body font-medium text-text">{stat.value}</span>
				<span class="text-caption text-text-3">{stat.label}</span>
			</span>
		</li>
	{/each}
</ul>
