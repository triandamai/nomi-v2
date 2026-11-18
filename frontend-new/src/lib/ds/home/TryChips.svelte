<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Landing Page → Option 3 — Hero Nomi → Try (14:645): the starters as pills, 8 apart and centred
	// under the composer: the owner's 16px mascot and the ask in 12px, padded 12 × 7 on the surface
	// with a quiet stroke. Picking one asks it; nothing is sent before that. They wrap on a phone.
	type Props = { items: { agent: CrewId; label: string }[]; onpick: (index: number) => void };
	let { items, onpick }: Props = $props();
</script>

{#if items.length > 0}
	<ul aria-label={m.starters_title()} class="flex flex-wrap justify-center gap-2">
		{#each items as item, index (item.label)}
			<li>
				<button
					type="button"
					onclick={() => onpick(index)}
					class="inline-flex cursor-pointer items-center gap-1.5 rounded-full border border-border bg-surface px-2.75 py-1.5 text-caption text-text-2 transition-colors outline-offset-2 hover:border-border-strong hover:text-text focus-visible:outline-2 focus-visible:outline-focus pointer-coarse:min-h-11"
				>
					<Mascot agent={item.agent} size={16} />
					{item.label}
				</button>
			</li>
		{/each}
	</ul>
{/if}
