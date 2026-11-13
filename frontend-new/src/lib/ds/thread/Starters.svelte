<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import SuggestionRow from '../lists/SuggestionRow.svelte';
	import SectionLabel from '../status/SectionLabel.svelte';
	import type { CrewId } from '../agents/crew';

	// App / Home → Suggestions (Ce004), and a blank chat's starters: "TRY", then two columns of
	// Suggestion Rows 10 apart, each routed to who owns it ("→ dana"). Picking one asks it; nothing
	// is sent before that. One column on a phone. The pen's rows wear mascots left over from earlier
	// copy (Koda's on "Summarize this week's spending → dana"): each wears its owner's here (§9).
	type Props = { items: { agent: CrewId; label: string; route: string }[]; onpick: (index: number) => void };
	let { items, onpick }: Props = $props();
</script>

{#if items.length > 0}
	<section class="flex w-full flex-col gap-2.5">
		<SectionLabel label={m.starters_title()} />
		<ul class="grid gap-2.5 md:grid-cols-2">
			{#each items as item, index (item.label)}
				<li class="min-w-0"><SuggestionRow agent={item.agent} label={item.label} route={item.route} onclick={() => onpick(index)} /></li>
			{/each}
		</ul>
	</section>
{/if}
