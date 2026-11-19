<script lang="ts">
	import type { Snippet } from 'svelte';
	import AgentByline from '../identity/AgentByline.svelte';
	import Byline from '../mobile/Byline.svelte';
	import VersionSwitcher from '../chat/VersionSwitcher.svelte';
	import type { CrewId } from '../agents/crew';

	// Flow / Chat — Ask → think → answer (ENb07) → Assistant Message: one reply in the thread, its
	// parts 16 apart. The byline says who and when in mono ("14:02 · 9s · stopped"), with the
	// version arrows once it was asked for again (Like & dislike, step 4: "‹ 2 / 2 ›"). Under it go
	// the reasoning, the prose or the live text, notes, the action bar and what opens from it, as
	// children. While the person edits the message it answers, the reply dims: it belongs to the
	// old version (Message controls, step 4). `phone` is a phone's (Mobile Chat → Answer): the
	// Mobile Byline (14, 600), the parts 12 apart, and dimmed to 0.35 (Message controls bRx3K).
	type Props = {
		agent: CrewId;
		name: string;
		meta?: string;
		version?: { current: number; total: number; onprev: () => void; onnext: () => void };
		dimmed?: boolean;
		phone?: boolean;
		children?: Snippet;
	};
	let { agent, name, meta, version, dimmed = false, phone = false, children }: Props = $props();
</script>

<article class={['flex w-full min-w-0 flex-col transition-opacity', phone ? 'gap-3' : 'gap-4', dimmed && (phone ? 'opacity-35' : 'opacity-40')]} aria-label={name}>
	<span class="flex flex-wrap items-center gap-2">
		{#if phone}<Byline {agent} {name} {meta} />{:else}<AgentByline {agent} {name} {meta} />{/if}
		{#if version}<VersionSwitcher {...version} />{/if}
	</span>
	{@render children?.()}
</article>
