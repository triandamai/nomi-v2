<script lang="ts">
	import { ChevronRight } from '@lucide/svelte';
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile — Home, Inbox & You, step 3 (O1SXw) → a settings group: its label in mono capitals,
	// then 52-high rows in a card (surface, border, radius 14): the icon at 18, the page at 16, a value in
	// mono where there is one, and a chevron.
	export type SettingsRow = { href: string; icon: Component<LucideProps>; label: string; meta?: string };
	type Props = { label: string; rows: SettingsRow[] };
	let { label, rows }: Props = $props();
	const id = $props.id();
</script>

<section aria-labelledby={id} class="flex flex-col gap-2">
	<h2 {id} class="px-1 font-mono text-label tracking-label text-text-3 uppercase">{label}</h2>
	<ul class="flex flex-col rounded-xl bg-surface outline -outline-offset-1 outline-border">
		{#each rows as row, index (row.href)}
			<li>
				<a href={row.href} class={['flex h-13 items-center gap-3 px-3.5 outline-offset-2 focus-visible:outline-2 focus-visible:outline-focus', index < rows.length - 1 && 'border-b border-border']}>
					<span class="inline-flex shrink-0 text-text-2"><Icon icon={row.icon} size={18} /></span>
					<span class="min-w-0 flex-1 truncate text-phone text-text">{row.label}</span>
					{#if row.meta}<span class="shrink-0 font-mono text-label text-text-3">{row.meta}</span>{/if}
					<span class="inline-flex shrink-0 text-text-3"><Icon icon={ChevronRight} size={14} /></span>
				</a>
			</li>
		{/each}
	</ul>
</section>
