<script lang="ts">
	import { Square } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '../actions/Button.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Chat responses → Banner / Crew Working: who's working on this chat right now, with one way to
	// stop them all. (The pen's button is named "Stop all"; its label was left at the template's.)
	type Props = { crew: CrewId[]; onstop?: () => void };
	let { crew, onstop }: Props = $props();
</script>

<div role="status" class="flex w-full flex-wrap items-center gap-2.5 rounded-sm py-1.5 pr-2 pl-3 outline -outline-offset-1 outline-border">
	<span class="flex items-center -space-x-1.5">
		{#each crew as agent (agent)}<Mascot {agent} size={20} />{/each}
	</span>
	<span class="min-w-0 flex-1 text-caption text-text">{crew.length === 1 ? m.crew_working_one() : m.crew_working({ count: String(crew.length) })}</span>
	{#if onstop}<Button label={m.crew_stop_all()} variant="ghost-danger" iconStart={Square} onclick={onstop} />{/if}
</div>
