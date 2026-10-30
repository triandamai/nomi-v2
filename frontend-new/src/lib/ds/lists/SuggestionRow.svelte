<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Something to ask, with who it would go to ("→ dana").
	type Props = { agent: CrewId; label: string; route?: string; onclick?: () => void };
	let { agent, label, route, onclick }: Props = $props();
</script>

<button type="button" {onclick} class="flex w-full cursor-pointer items-center gap-3 rounded-sm border border-border px-3 py-2.5 text-left outline-offset-1 hover:border-border-strong hover:bg-surface">
	<Mascot {agent} size={20} />
	<span class="min-w-0 flex-1 text-ui text-text">{label}</span>
	{#if route}<span class="font-mono text-label text-text-3">{route}</span>{/if}
</button>
