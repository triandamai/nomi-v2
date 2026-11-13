<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Info } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// App / Settings / Memory (hANTE) → Topic Table: the topics in an outlined table, its column names
	// in mono capitals over a stroke, then each topic's row, and a quiet note at the foot about how
	// strength works.
	type Props = { columns: { topic: string; knows: string; strength: string; updated: string }; note: string; children: Snippet };
	let { columns, note, children }: Props = $props();
</script>

<div class="flex flex-col overflow-hidden rounded-md outline -outline-offset-1 outline-border">
	<div aria-hidden="true" class="hidden gap-4 border-b border-border px-3.5 pt-2 pb-1.75 font-mono text-label tracking-label text-text-3 uppercase md:flex">
		<span class="w-30 shrink-0">{columns.topic}</span>
		<span class="min-w-0 flex-1">{columns.knows}</span>
		<span class="w-23 shrink-0">{columns.strength}</span>
		<span class="w-14 shrink-0">{columns.updated}</span>
		<span class="w-4 shrink-0"></span>
	</div>
	<ul class="flex flex-col">{@render children()}</ul>
	<p class="flex items-start gap-2 px-3.5 py-2 text-caption text-text-3">
		<span class="mt-0.5 inline-flex shrink-0"><Icon icon={Info} size={12} /></span>{note}
	</p>
</div>
