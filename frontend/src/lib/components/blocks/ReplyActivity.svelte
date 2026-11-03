<script lang="ts">
	// How a reply came about, in the design system's pieces: the crew's thinking (Reasoning, or
	// Thought when it's one short decision), the real steps it took with their tool and target
	// (Thought steps), and the pages it used (Sources). A command's output stays one click away.
	import { m } from '$lib/paraglide/messages';
	import { stepLabel } from '$lib/agentLabels';
	import { hostOf } from '$lib/citations';
	import Reasoning from '$lib/components/ds/chat/Reasoning.svelte';
	import Thought from '$lib/components/ds/chat/Thought.svelte';
	import ThoughtSteps from '$lib/components/ds/chat/ThoughtSteps.svelte';
	import Sources from '$lib/components/ds/chat/Sources.svelte';
	import CodeBlock from '$lib/components/ds/chat/CodeBlock.svelte';
	import type { ReplySource, ReplyStep } from '$lib/types';

	let {
		thinking = [],
		steps = [],
		sources = [],
		live = false,
	}: {
		/** Thinking, oldest first. */
		thinking?: string[];
		steps?: ReplyStep[];
		/** Pages the crew read or found, when the reply doesn't cite them itself. */
		sources?: ReplySource[];
		/** No reply yet: the agent may still be thinking. */
		live?: boolean;
	} = $props();

	const uid = $props.id();
	const thought = $derived(thinking.join('\n\n').trim());
	// One short decision reads as a Thought; anything longer folds away as Reasoning.
	const brief = $derived(!live && thinking.length === 1 && thought.length <= 200 && !thought.includes('\n'));
	let open = $state(false);
	const failed = $derived(steps.filter((step) => !step.ok).length);
	const outputs = $derived(steps.flatMap((step) => (step.output ? [{ label: stepLabel(step.tool, step.detail), output: step.output }] : [])));
</script>

<div class="reply-activity">
	{#if thought}
		{#if brief}
			<Thought text={thought} />
		{:else}
			<Reasoning label={live ? m.activity_thinking() : m.reasoning_process()} text={thought} {live} />
		{/if}
	{/if}
	{#if steps.length > 0}
		<ThoughtSteps
			bind:open
			label={steps.length === 1 ? m.activity_step_one() : m.activity_steps({ count: steps.length })}
			meta={failed > 0 ? m.activity_failed() : undefined}
			steps={steps.map((step) => ({ label: stepLabel(step.tool, step.detail), meta: step.detail && step.tool !== 'delegate_to_agent' ? step.detail : undefined, state: step.ok ? 'done' : 'failed' }))}
		/>
		{#if open}
			{#each outputs as item, index (index)}
				<details class="reply-activity__output">
					<summary>{m.activity_show_output()} · {item.label}</summary>
					<CodeBlock code={item.output} file={item.label} />
				</details>
			{/each}
		{/if}
	{/if}
	{#if sources.length > 0}
		<Sources id="{uid}-sources" sources={sources.map((source) => ({ title: source.title ?? hostOf(source.url), detail: hostOf(source.url), kind: 'web' as const, href: source.url }))} />
	{/if}
</div>

<style>
	.reply-activity {
		display: flex;
		flex-direction: column;
		gap: 8px;
		width: 100%;
		margin-bottom: 8px;
	}
	.reply-activity__output summary {
		cursor: pointer;
		margin-bottom: 6px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
</style>
