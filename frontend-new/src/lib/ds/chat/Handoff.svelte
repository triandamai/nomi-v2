<script lang="ts">
	import { ArrowRight } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Chat responses → Handoff: who passed the question to whom, and why, so the answer's owner is
	// never a surprise. You can send it back.
	type Member = { agent: CrewId; name: string };
	type Props = { from: Member; to: Member; reason?: string; onback?: () => void };
	let { from, to, reason, onback }: Props = $props();
</script>

<p class="flex w-full flex-wrap items-center gap-2 rounded-sm border border-border px-3 py-2">
	<Mascot agent={from.agent} size={20} />
	<span class="inline-flex text-text-3"><Icon icon={ArrowRight} size={12} /></span>
	<Mascot agent={to.agent} size={20} />
	<span class="text-caption text-text-2">{m.handoff({ from: from.name, to: to.name })}</span>
	<span class="min-w-0 flex-1 font-mono text-label text-text-3">{reason}</span>
	{#if onback}
		<button type="button" class="cursor-pointer rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={onback}>{m.handoff_back({ name: from.name })}</button>
	{/if}
</p>
