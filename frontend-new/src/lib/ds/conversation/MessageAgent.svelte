<script lang="ts">
	import type { Snippet } from 'svelte';
	import AgentByline from '../identity/AgentByline.svelte';
	import type { CrewId } from '../agents/crew';

	// An answer: who gave it and when ("14:02 · deepseek-flash"), then prose first. Rich blocks
	// (tables, plans, sources) come as children, only when the answer has that shape.
	type Props = { agent: CrewId; name: string; meta?: string; text?: string; children?: Snippet };
	let { agent, name, meta, text, children }: Props = $props();
</script>

<article class="flex w-full flex-col gap-3">
	<AgentByline {agent} {name} {meta} />
	{#if text}<p class="text-body leading-relaxed whitespace-pre-wrap text-text-2">{text}</p>{/if}
	{@render children?.()}
</article>
