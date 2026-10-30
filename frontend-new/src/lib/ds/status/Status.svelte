<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Design System / Nomi → Status: a 6px dot plus a word, so colour never carries meaning alone.
	// Who is working shows their mascot instead of a dot. States, not decoration: no pulsing.
	type Props = { kind: 'live' | 'ready' | 'idle' | 'failed' | 'attention' | 'working'; label: string; agent?: CrewId };
	let { kind, label, agent }: Props = $props();
</script>

<span class="inline-flex items-center gap-1.5">
	{#if kind === 'working' && agent}
		<Mascot {agent} size={20} />
	{:else}
		<span
			data-dot
			class={[
				'size-1.5 shrink-0 rounded-full',
				(kind === 'live' || kind === 'ready') && 'bg-accent',
				kind === 'idle' && 'bg-text-3',
				kind === 'failed' && 'bg-danger',
				kind === 'attention' && 'bg-warning',
			]}
		></span>
	{/if}
	<span
		class={[
			kind === 'ready' || kind === 'idle' ? 'font-mono text-label' : 'text-caption',
			(kind === 'live' || kind === 'working') && 'text-text',
			kind === 'ready' && 'text-text-2',
			kind === 'idle' && 'text-text-3',
			kind === 'failed' && 'text-danger',
			kind === 'attention' && 'text-warning',
		]}>{label}</span
	>
</span>
