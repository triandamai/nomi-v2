<script lang="ts">
	import { ChevronDown, ChevronRight, ListTree } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import { STEP_ICON, type StepState } from './stepState';

	// Chat responses → Thought steps: the real steps behind an answer, each with its tool, size and
	// time in mono ("dana.records · 142 rows · 1.1s").
	type Step = { label: string; meta?: string; state: StepState };
	type Props = { label: string; meta?: string; steps: Step[]; open?: boolean };
	let { label, meta, steps, open = $bindable(true) }: Props = $props();

	const id = $props.id();
</script>

<div class="flex w-full flex-col gap-2.5">
	<button
		type="button"
		aria-expanded={open}
		aria-controls={id}
		onclick={() => (open = !open)}
		class="inline-flex cursor-pointer items-center gap-1.5 self-start rounded-xs py-1 text-ui text-text-2 outline-offset-2 hover:text-text"
	>
		<span class="inline-flex text-text-3"><Icon icon={ListTree} size={14} /></span>
		{label}
		{#if meta}<span class="font-mono text-label text-text-3">{meta}</span>{/if}
		<span class="inline-flex text-text-3"><Icon icon={open ? ChevronDown : ChevronRight} size={12} /></span>
	</button>
	{#if open}
		<ul {id} class="flex flex-col gap-3 border-l border-border-strong py-0.5 pl-3.25">
			{#each steps as step, index (index)}
				<li class="flex gap-2.5">
					<span
						class={[
							'inline-flex shrink-0 items-start pt-0.5',
							step.state === 'done' && 'text-accent',
							step.state === 'running' && 'text-text-2',
							step.state === 'waiting' && 'text-text-3',
							step.state === 'failed' && 'text-danger',
						]}
						><span class={['inline-flex', step.state === 'running' && 'animate-spin motion-reduce:animate-none']}><Icon icon={STEP_ICON[step.state]} size={14} /></span></span
					>
					<span class="flex min-w-0 flex-col gap-0.5">
						<span class={['text-ui', step.state === 'running' ? 'text-text' : 'text-text-2']}>{step.label}</span>
						{#if step.meta}<span class="font-mono text-label text-text-3">{step.meta}</span>{/if}
					</span>
				</li>
			{/each}
		</ul>
	{/if}
</div>
