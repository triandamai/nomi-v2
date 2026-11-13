<script lang="ts">
	import type { Snippet } from 'svelte';
	import { ChevronDown, ChevronRight } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import StrengthBars from './StrengthBars.svelte';

	// App / Settings / Memory (hANTE) → Topic: one topic of what the crew remembers, [10,14], 16
	// apart: its name and how many memories, what it knows in a line or two, its strength and when
	// it last changed. A fading topic is quieter. It opens to its memories.
	type Props = {
		name: string;
		count: string;
		summary: string;
		strength: { level: 1 | 2 | 3 | 4 | 5; name: string };
		updated: string;
		fading?: boolean;
		open?: boolean;
		children?: Snippet;
	};
	let { name, count, summary, strength, updated, fading = false, open = $bindable(false), children }: Props = $props();

	const id = $props.id();
</script>

<li class="border-b border-border">
	<button
		type="button"
		aria-expanded={open}
		aria-controls="{id}-memories"
		onclick={() => (open = !open)}
		class="flex w-full cursor-pointer flex-wrap items-center gap-x-4 gap-y-1 px-3.5 pt-2.5 pb-2.25 text-left outline-offset-1 hover:bg-surface md:flex-nowrap"
	>
		<span class="flex w-30 shrink-0 flex-col gap-0.5">
			<span class={['text-ui font-medium', fading ? 'text-text-2' : 'text-text']}>{name}</span>
			<span class="font-mono text-label text-text-3">{count}</span>
		</span>
		<span class={['line-clamp-2 min-w-0 flex-1 text-caption', fading ? 'text-text-3' : 'text-text-2']}>{summary}</span>
		<span class="w-23 shrink-0"><StrengthBars level={strength.level} name={strength.name} /></span>
		<span class="w-14 shrink-0 font-mono text-label text-text-3">{updated}</span>
		<span class="inline-flex w-4 shrink-0 text-text-3"><Icon icon={open ? ChevronDown : ChevronRight} size={14} /></span>
	</button>
	{#if open && children}<ul id="{id}-memories" class="flex flex-col border-t border-border bg-surface">{@render children()}</ul>{/if}
</li>
