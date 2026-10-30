<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Design System / Nomi → Identity: who is speaking, and when (in mono: "15:37 · 4m 12s").
	let { agent, name, meta }: { agent: CrewId; name: string; meta?: string } = $props();
</script>

<span class="inline-flex items-center gap-2">
	<Mascot {agent} size={20} />
	<span class="text-ui font-medium text-text">{name}</span>
	{#if meta}<span class="font-mono text-label text-text-3">{meta}</span>{/if}
</span>
