<script lang="ts">
	import { ListChecks, Square } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';
	import { STEP_ICON, stepNumber, type StepState } from './stepState';

	// Chat responses → Plan (running): numbered steps with their real state, what each produced or
	// waits for, and the crew member who owns it; how far along it is; a way to stop.
	type Step = { label: string; meta?: string; state: StepState; owner: { agent: CrewId; name: string } };
	type Props = { subject: string; steps: Step[]; onstop?: () => void };
	let { subject, steps, onstop }: Props = $props();

	const done = $derived(steps.filter((step) => step.state === 'done').length);
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md bg-bg outline -outline-offset-1 outline-border-strong">
	<header class="flex items-center gap-2.5 border-b border-border bg-surface px-3.5 pt-2.5 pb-2.25">
		<span class="inline-flex text-text-2"><Icon icon={ListChecks} size={14} /></span>
		<span class="text-ui font-medium text-text">{m.plan_title()}</span>
		<span class="min-w-0 flex-1 truncate text-ui text-text-2">{subject}</span>
		<span class="font-mono text-label text-text-3">{m.plan_progress({ done: String(done), total: String(steps.length) })}</span>
	</header>
	<ol>
		{#each steps as step, index (index)}
			<li class={['flex items-center gap-3 px-3.5 pb-2.5', index > 0 ? 'border-t border-border pt-2.25' : 'pt-2.5']}>
				<span class="font-mono text-label text-text-3">{stepNumber(index)}</span>
				<span
					class={[
						'inline-flex shrink-0',
						step.state === 'done' && 'text-accent',
						step.state === 'running' && 'animate-spin text-text motion-reduce:animate-none',
						step.state === 'waiting' && 'text-text-3',
						step.state === 'failed' && 'text-danger',
					]}><Icon icon={STEP_ICON[step.state]} size={14} /></span
				>
				<span class="flex min-w-0 flex-1 flex-col gap-0.5">
					<span class={['text-ui', step.state === 'waiting' ? 'text-text-2' : 'text-text']}>{step.label}</span>
					{#if step.meta}<span class="font-mono text-label text-text-3">{step.meta}</span>{/if}
				</span>
				<span class="inline-flex shrink-0 items-center gap-1.5">
					<Mascot agent={step.owner.agent} size={20} />
					<span class="text-caption text-text-2">{step.owner.name}</span>
				</span>
			</li>
		{/each}
	</ol>
	<footer class="flex items-center gap-2 border-t border-border pt-1.25 pr-2 pb-1.5 pl-3.5">
		<span class="flex-1 text-caption text-text-3">{m.plan_note()}</span>
		{#if onstop}<Button label={m.plan_stop()} variant="ghost" iconStart={Square} onclick={onstop} />{/if}
	</footer>
</section>
