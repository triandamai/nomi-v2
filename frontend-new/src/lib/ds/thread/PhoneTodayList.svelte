<script lang="ts">
	import { ChevronRight } from '@lucide/svelte';
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile — Home, Inbox & You, step 1 (dl13j) → Today (GjEzd): what moved since the person
	// last looked, one 64-high row each in a card (surface, border, radius 14): a 36 tile (radius 10)
	// with its icon at 18, the title at 16, one line under it at 13, the count in mono and a chevron.
	// Each row opens where its things are.
	export type TodayRow = { icon: Component<LucideProps>; title: string; detail: string; count: number; href: string };
	type Props = { rows: TodayRow[]; label: string };
	let { rows, label }: Props = $props();
</script>

<nav aria-label={label} class="flex flex-col rounded-xl bg-surface outline -outline-offset-1 outline-border">
	{#each rows as row, index (row.href + row.title)}
		<a href={row.href} class={['flex h-16 items-center gap-3 px-3.5 outline-offset-2 focus-visible:outline-2 focus-visible:outline-focus', index < rows.length - 1 && 'border-b border-border']}>
			<span class="inline-flex size-9 shrink-0 items-center justify-center rounded-md bg-surface-2 text-text-2"><Icon icon={row.icon} size={18} /></span>
			<span class="flex min-w-0 flex-1 flex-col gap-0.5">
				<span class="truncate text-phone text-text">{row.title}</span>
				<span class="truncate text-ui text-text-3">{row.detail}</span>
			</span>
			<span class="shrink-0 font-mono text-label text-text-2">{row.count}</span>
			<span class="inline-flex shrink-0 text-text-3"><Icon icon={ChevronRight} size={14} /></span>
		</a>
	{/each}
</nav>
