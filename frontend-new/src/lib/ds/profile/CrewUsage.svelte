<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// App / Settings / Profile (U9Gnrg) → Your crew: who the person works with, each with their
	// mascot, name, a bar in their own colour against the busiest, and how much in mono, 12 apart.
	export type CrewUse = { agent: CrewId; name: string; count: string; share: number };
	type Props = { crew: CrewUse[]; empty: string };
	let { crew, empty }: Props = $props();
</script>

{#if crew.length}
	<ul class="flex flex-col gap-3">
		{#each crew as member (member.agent)}
			<li class="flex items-center gap-3">
				<Mascot agent={member.agent} size={20} />
				<span class="w-14 shrink-0 truncate text-ui text-text">{member.name}</span>
				<span class="flex min-w-0 flex-1">
					<span class="h-2 rounded-xs" style="width: {Math.max(4, Math.round(Math.min(1, member.share) * 100))}%; background: var(--crew-{member.agent}-bg)"></span>
				</span>
				<span class="shrink-0 font-mono text-label text-text-2">{member.count}</span>
			</li>
		{/each}
	</ul>
{:else}
	<p class="text-caption text-text-3">{empty}</p>
{/if}
