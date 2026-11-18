<script lang="ts">
	import type { Component, Snippet } from 'svelte';
	import { ChevronRight, type LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Mobile Build → one row of a sheet's list: Deploy / #2 (the number in mono, the title at
	// 15, its meta in mono, the state or Roll back at the end; [12,14], 10 apart), Order (title and
	// total), File (an 18 icon, the path in mono at 13, its meta, a chevron; 52 tall, [0,14], 12
	// apart). It opens something when it has `onclick` (a button with the chevron), else it's a row.
	type Props = {
		title: string;
		meta?: string;
		/** A machine's value before the title (`#2`). */
		lead?: string;
		icon?: Component<LucideProps>;
		/** The title is machine output (a path, a table's value). */
		mono?: boolean;
		/** Pressed state, for a row that's on (an open folder). */
		current?: boolean;
		/** What leads the row where an icon isn't enough: a person's face. */
		start?: Snippet;
		end?: Snippet;
		onclick?: () => void;
	};
	let { title, meta, lead, icon, mono = false, current, start, end, onclick }: Props = $props();
</script>

{#snippet inside()}
	{#if lead}<span class="w-7 shrink-0 font-mono text-ui text-text">{lead}</span>{/if}
	{#if start}{@render start()}{/if}
	{#if icon}<span class="inline-flex shrink-0 text-text-2"><Icon {icon} size={18} /></span>{/if}
	<span class="flex min-w-0 flex-1 flex-col gap-0.5">
		<span class={['break-words text-text', mono ? 'font-mono text-ui' : 'text-body']}>{title}</span>
		{#if meta}<span class="font-mono text-label break-words text-text-3">{meta}</span>{/if}
	</span>
	{#if end}{@render end()}{/if}
	{#if onclick}<span class="inline-flex shrink-0 text-text-3"><Icon icon={ChevronRight} size={14} /></span>{/if}
{/snippet}

<li class="flex border-b border-border last:border-b-0">
	{#if onclick}
		<button type="button" aria-expanded={current} class={['flex min-h-13 w-full cursor-pointer items-center px-3.5 py-2 text-left -outline-offset-2', icon || start ? 'gap-3' : 'gap-2.5']} {onclick}>
			{@render inside()}
		</button>
	{:else}
		<div class={['flex min-h-13 w-full items-center px-3.5 py-3', icon || start ? 'gap-3' : 'gap-2.5']}>
			{@render inside()}
		</div>
	{/if}
</li>
