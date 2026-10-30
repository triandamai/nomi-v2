<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Mobile / Byline: who answered, at 14pt, and when, in mono.
	type Props = { agent: CrewId; name: string; meta?: string };
	let { agent, name, meta }: Props = $props();
</script>

<p class="flex items-center gap-2">
	<Mascot {agent} size={20} />
	<span class="text-phone-sm font-semibold text-text">{name}</span>
	{#if meta}<span class="font-mono text-label text-text-3">{meta}</span>{/if}
</p>
