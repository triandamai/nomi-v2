<script lang="ts">
	import { ChevronRight } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Home's crew, folded into one pill under the top bar's search (the person's call: the crew
	// leaves the sidebar the Landing Page draws, 14:515, and the old crew column, EXxti). Up to five
	// faces overlapping by 6 (each on a ring of the page so the edges read, as the design's "More
	// crew" row draws them), then the crew's state in mono: the accent dot while someone works, a
	// quiet one when all are idle. It opens the crew drawer; `open` is its expanded state.
	type Props = { faces: CrewId[]; summary: { kind: 'idle' | 'live'; label: string }; label: string; open?: boolean; onclick: () => void };
	let { faces, summary, label, open = false, onclick }: Props = $props();
</script>

<button
	type="button"
	{onclick}
	aria-haspopup="dialog"
	aria-expanded={open}
	aria-label="{label} · {summary.label}"
	class="inline-flex cursor-pointer items-center gap-2 rounded-full border border-border bg-surface py-1 pr-2.25 pl-1.25 text-text-2 transition-colors outline-offset-2 hover:border-border-strong hover:text-text focus-visible:outline-2 focus-visible:outline-focus pointer-coarse:min-h-11"
>
	<span aria-hidden="true" class="flex items-center">
		{#each faces.slice(0, 5) as agent, index (agent)}
			<span class={['inline-flex size-6 items-center justify-center rounded-full bg-surface', index > 0 && '-ml-1.5']}><Mascot {agent} size={20} live={false} /></span>
		{/each}
	</span>
	<span aria-hidden="true" class="inline-flex items-center gap-1.5 font-mono text-label">
		<span class={['size-1.5 shrink-0 rounded-full', summary.kind === 'live' ? 'bg-accent' : 'bg-text-3']}></span>
		<span class={summary.kind === 'live' ? 'text-text' : 'text-text-3'}>{summary.label}</span>
	</span>
	<span aria-hidden="true" class="text-text-3"><Icon icon={ChevronRight} size={14} /></span>
</button>
