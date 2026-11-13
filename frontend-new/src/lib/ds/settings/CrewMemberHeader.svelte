<script lang="ts">
	import type { Snippet } from 'svelte';
	import CrewMark from './CrewMark.svelte';
	import Tag from '../status/Tag.svelte';
	import type { CrewId } from '../agents/crew';

	// App / Settings / Crew (vVUeh) → Header: the chosen member, on their own tint (the surface for a
	// custom one), 16 round, [20,24] and 14 apart: the 32px mark, the name (15/500) with its group as
	// a tag, and what it does in mono under it; `tools` on the right.
	type Props = { agent?: CrewId; initial?: string; name: string; tag: string; detail: string; tools?: Snippet };
	let { agent, initial, name, tag, detail, tools }: Props = $props();
</script>

<header
	style={agent ? `background: var(--crew-${agent}-bg)` : undefined}
	class={['flex items-center gap-3.5 rounded-2xl px-6 py-5', !agent && 'bg-surface']}
>
	<CrewMark {agent} initial={initial ?? name.charAt(0)} size={32} />
	<div class="flex min-w-0 flex-1 flex-col gap-0.5">
		<div class="flex items-center gap-2">
			<h2 class="truncate text-body font-medium text-text">{name}</h2>
			<Tag kind="neutral" label={tag} />
		</div>
		<p class="truncate font-mono text-label text-text-3">{detail}</p>
	</div>
	{@render tools?.()}
</header>
