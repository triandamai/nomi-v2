<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Flow / Chat — Errors & recovery (TpuRc): what happened to a turn, kept quietly in the thread.
	// `line` is a mono note with a 12px icon, 6 apart ("continued after the connection dropped at
	// 18s", "sent after you signed in · 10:31"); `box` folds a recovered failure into one bordered
	// row, padded [8,12], 8 apart, with a 14px icon and an optional action ("First try failed: … ·
	// Details", "Rate limited by DeepSeek for 24s, then retried automatically.").
	type Props = {
		icon: Component<LucideProps>;
		text: string;
		kind?: 'line' | 'box';
		align?: 'start' | 'end';
		tone?: 'quiet' | 'danger';
		action?: { label: string; onclick: () => void };
	};
	let { icon, text, kind = 'line', align = 'start', tone = 'quiet', action }: Props = $props();
</script>

{#if kind === 'box'}
	<p class="flex w-full items-center gap-2 rounded-sm px-3 py-2 outline -outline-offset-1 outline-border">
		<span class="inline-flex shrink-0 text-text-3"><Icon {icon} size={14} /></span>
		<span class="min-w-0 flex-1 text-caption text-text-2">{text}</span>
		{#if action}<button type="button" class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={action.onclick}>{action.label}</button>{/if}
	</p>
{:else}
	<p class={['flex items-center gap-1.5 font-mono text-label', align === 'end' && 'justify-end self-end text-right', tone === 'danger' ? 'text-danger' : 'text-text-3']}>
		<span class="inline-flex shrink-0"><Icon {icon} size={12} /></span>
		<span class="min-w-0">{text}</span>
		{#if action}<button type="button" class="cursor-pointer rounded-xs font-sans text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={action.onclick}>{action.label}</button>{/if}
	</p>
{/if}
