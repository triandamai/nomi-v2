<script lang="ts">
	import type { Component } from 'svelte';
	import { ChevronRight, type LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Mobile / Sheet Row: one action in a bottom sheet, 52pt tall, 16pt, with a note in mono or a
	// chevron when it opens more.
	type Props = { icon: Component<LucideProps>; label: string; meta?: string; more?: boolean; danger?: boolean; onclick: () => void };
	let { icon, label, meta, more = false, danger = false, onclick }: Props = $props();
</script>

<button type="button" class="flex h-13 w-full cursor-pointer items-center gap-3.5 px-5 text-left -outline-offset-2" {onclick}>
	<span class={['inline-flex shrink-0', danger ? 'text-danger' : 'text-text-2']}><Icon {icon} size={20} /></span>
	<span class={['min-w-0 flex-1 truncate text-phone', danger ? 'text-danger' : 'text-text']}>{label}</span>
	{#if meta}<span class="font-mono text-label text-text-3">{meta}</span>{/if}
	{#if more}<span class="inline-flex text-text-3"><Icon icon={ChevronRight} /></span>{/if}
</button>
