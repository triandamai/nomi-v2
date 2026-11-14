<script lang="ts">
	import type { Snippet } from 'svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Flow / Build → Screen / Building (H4DIH) → Chat Header: the agent chat's head, 44 high,
	// [0,12,0,20] and 10 apart over its line: "Chat" (13/500), who's on it (20px mascots, 4 apart)
	// with what they're doing in mono, and `tools` at the end (the pen's history and collapse).
	type Props = { title: string; crew: CrewId[]; note?: string; tools?: Snippet };
	let { title, crew, note, tools }: Props = $props();
</script>

<div class="flex h-11 shrink-0 items-center gap-2.5 border-b border-border pr-3 pl-5">
	<span class="text-ui font-medium text-text">{title}</span>
	<span class="flex min-w-0 flex-1 items-center gap-1">
		{#each crew as agent (agent)}<Mascot {agent} size={20} />{/each}
		{#if note}<span class="truncate font-mono text-label text-text-3">{note}</span>{/if}
	</span>
	{@render tools?.()}
</div>
