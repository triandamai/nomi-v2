<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Flow / Mobile Chat — Start & answer → New chat (step 2, kEp8c / x0y5RK): the starters under the
	// line, one column 8 apart. Each row is the Suggestion Row with its label at 15 (`text`) instead of
	// 13, so a long one wraps to two lines (the row 58 tall); the owner's mascot (20) and "→ koda" in
	// mono as on a desktop. Picking one asks it; nothing is sent before that. SuggestionRow is the old
	// app's twin and keeps its 13, so the phone's rows are their own.
	type Props = { items: { agent: CrewId; label: string; route: string }[]; onpick: (index: number) => void };
	let { items, onpick }: Props = $props();
</script>

{#if items.length > 0}
	<ul class="flex w-full flex-col gap-2 pt-2">
		{#each items as item, index (item.label)}
			<li>
				<button
					type="button"
					class="flex w-full cursor-pointer items-center gap-3 rounded-sm border border-border px-2.75 py-2.25 text-left outline-offset-1 hover:border-border-strong hover:bg-surface"
					onclick={() => onpick(index)}
				>
					<Mascot agent={item.agent} size={20} live />
					<span class="min-w-0 flex-1 text-body text-text">{item.label}</span>
					<span class="font-mono text-label text-text-3">{item.route}</span>
				</button>
			</li>
		{/each}
	</ul>
{/if}
