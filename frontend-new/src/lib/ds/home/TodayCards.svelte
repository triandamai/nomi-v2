<script lang="ts" module>
	import type { CrewId } from '../agents/crew';

	export type TodayCard = {
		key: 'out' | 'reminders' | 'plans';
		agent: CrewId;
		title: string;
		body: string;
		action: { label: string; href?: string; onclick?: () => void };
	};
</script>

<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';

	// Landing Page → Option 3 — Hero Nomi → Today (14:670): three cards, 12 apart: the mascot of
	// who it's about (28px), then the title (13, medium) over one line of what's there (12, 1.4),
	// padded 14 × 12 with a quiet stroke and 16 corners. The whole card is the way in (the Inbox,
	// a plan, or a prompt in the composer); its action's words are its accessible description.
	// A phone stacks them.
	type Props = { cards: TodayCard[] };
	let { cards }: Props = $props();

	const id = $props.id();
</script>

{#snippet inner(card: TodayCard)}
	<Mascot agent={card.agent} size={28} />
	<span class="flex min-w-0 flex-1 flex-col gap-0.5 text-left">
		<span class="text-ui font-medium text-text">{card.title}</span>
		<span class="text-caption leading-cozy text-text-2">{card.body}</span>
	</span>
	<span id="{id}-{card.key}" hidden>{card.action.label}</span>
{/snippet}

{#if cards.length > 0}
	<ul class="grid w-full gap-3 md:grid-cols-3">
		{#each cards as card (card.key)}
			<li class="min-w-0">
				{#if card.action.href}
					<a href={card.action.href} aria-describedby="{id}-{card.key}" class="flex h-full w-full items-center gap-3 rounded-2xl bg-surface px-3.5 py-3 outline -outline-offset-1 outline-border transition-colors hover:bg-surface-2 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus">{@render inner(card)}</a>
				{:else}
					<button type="button" onclick={card.action.onclick} aria-describedby="{id}-{card.key}" class="cursor-pointer flex h-full w-full items-center gap-3 rounded-2xl bg-surface px-3.5 py-3 outline -outline-offset-1 outline-border transition-colors hover:bg-surface-2 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-focus">{@render inner(card)}</button>
				{/if}
			</li>
		{/each}
	</ul>
{/if}
