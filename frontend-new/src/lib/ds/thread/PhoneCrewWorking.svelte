<script lang="ts">
	import { Square } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Button from '../actions/Button.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Flow / Mobile Chat — Approvals & background, step 4 (HsE8N): who is working on this chat while
	// the person may leave, under the nav bar: the pen's Chat / Banner / Crew Working with the
	// phone's words ("3 crew working") and Stop all, which stops them all ("stop"). The pen's
	// button reads "Team and invites", pasted from another frame (AGENTS.md §9).
	type Props = { crew: CrewId[]; onstop?: () => void };
	let { crew, onstop }: Props = $props();
</script>

<div role="status" class="flex w-full items-center gap-2.5 rounded-sm py-1.5 pr-2 pl-3 outline -outline-offset-1 outline-border">
	<span class="flex items-center -space-x-1.5">
		{#each crew as agent, index (`${agent}-${index}`)}<Mascot {agent} size={20} />{/each}
	</span>
	<span class="min-w-0 flex-1 text-caption text-text">{m.phone_crew_working({ count: String(crew.length) })}</span>
	{#if onstop}<Button label={m.crew_stop_all()} variant="ghost-danger" iconStart={Square} onclick={onstop} />{/if}
</div>
