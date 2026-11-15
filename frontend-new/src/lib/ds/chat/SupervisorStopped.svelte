<script lang="ts">
	import { Check, CircleDashed, Play, Square } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Chat responses → Supervisor Stopped: what was stopped, task by task: done before the stop,
	// stopped part-way (and how far it got), or never started. What already left Nomi isn't undone.
	type Task = { agent: CrewId; title: string; detail?: string; state: 'done' | 'stopped' | 'waiting' };
	type Props = { meta?: string; tasks: Task[]; ondiscard?: () => void; onresume?: () => void };
	let { meta, tasks, ondiscard, onresume }: Props = $props();

	const ICON = { done: Check, stopped: Square, waiting: CircleDashed } as const;
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md bg-bg outline -outline-offset-1 outline-border-strong">
	<header class="flex items-center gap-2.5 border-b border-border bg-surface px-3.5 pt-2.5 pb-2.25">
		<Mascot agent="supervisor" size={20} />
		<p class="min-w-0 flex-1 text-ui font-medium text-text">{tasks.length === 1 ? m.supervisor_stopped_one() : m.supervisor_stopped({ count: String(tasks.length) })}</p>
		{#if meta}<span class="font-mono text-label text-text-3">{meta}</span>{/if}
	</header>
	<ul class="px-3.5 py-1">
		{#each tasks as task, index (index)}
			<li class={['flex flex-wrap items-center gap-2.5 pb-2', index > 0 ? 'border-t border-border pt-1.75' : 'pt-2']}>
				<span class={['inline-flex shrink-0', task.state === 'done' ? 'text-accent' : 'text-text-3']}><Icon icon={ICON[task.state]} size={14} /></span>
				<Mascot agent={task.agent} size={20} />
				<span class="min-w-0 flex-1 text-ui text-text">{task.title}</span>
				{#if task.detail}<span class="font-mono text-label text-text-3">{task.detail}</span>{/if}
			</li>
		{/each}
	</ul>
	<footer class="flex flex-wrap items-center gap-2 border-t border-border px-3.5 pt-2.25 pb-2.5">
		<span class="min-w-0 flex-1 basis-40 text-caption text-text-3">{m.supervisor_note()}</span>
		<span class="ml-auto flex items-center gap-2">
			{#if ondiscard}<Button label={m.supervisor_discard()} variant="ghost" onclick={ondiscard} />{/if}
			{#if onresume}<Button label={m.supervisor_resume()} variant="secondary" iconEnd={Play} onclick={onresume} />{/if}
		</span>
	</footer>
</section>
