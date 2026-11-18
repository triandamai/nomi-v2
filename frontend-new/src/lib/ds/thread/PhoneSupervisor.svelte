<script lang="ts">
	import { Check, CircleDashed, Play, Square } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Flow / Mobile Chat — Approvals & background, step 6 (nQiZd): what Stop all stopped, task by
	// task, as the pen's Chat / Supervisor Stopped with the phone flow's words: each task's state in
	// mono at its end ("done", "stopped · 2 of 3", "not started") and "Nothing sent was undone.".
	// Resume picks up again; the pen's Discard is left out (nothing in the backend discards).
	type Task = { agent: CrewId; title: string; detail?: string; state: 'done' | 'stopped' | 'waiting' };
	type Props = { tasks: Task[]; onresume?: () => void };
	let { tasks, onresume }: Props = $props();

	const ICON = { done: Check, stopped: Square, waiting: CircleDashed } as const;
	const stateLabel = (task: Task) =>
		task.state === 'done' ? m.phone_supervisor_done() : task.state === 'waiting' ? m.phone_supervisor_not_started() : task.detail ? m.phone_supervisor_stopped({ detail: task.detail }) : m.phone_supervisor_stopped_plain();
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md bg-bg outline -outline-offset-1 outline-border-strong">
	<header class="flex items-center gap-2.5 border-b border-border bg-surface px-3.5 pt-2.5 pb-2.25">
		<Mascot agent="supervisor" size={20} />
		<p class="min-w-0 flex-1 text-ui font-medium text-text">{tasks.length === 1 ? m.supervisor_stopped_one() : m.supervisor_stopped({ count: String(tasks.length) })}</p>
	</header>
	<ul class="px-3.5 py-1">
		{#each tasks as task, index (index)}
			<li class={['flex items-center gap-2.5 pb-2', index > 0 ? 'border-t border-border pt-1.75' : 'pt-2']}>
				<span class={['inline-flex shrink-0', task.state === 'done' ? 'text-accent' : 'text-text-3']}><Icon icon={ICON[task.state]} size={14} /></span>
				<Mascot agent={task.agent} size={20} />
				<span class="min-w-0 flex-1 text-ui text-text">{task.title}</span>
				<span class="shrink-0 font-mono text-label text-text-3">{stateLabel(task)}</span>
			</li>
		{/each}
	</ul>
	<footer class="flex flex-wrap items-center gap-2 border-t border-border px-3.5 pt-2.25 pb-2.5">
		<span class="min-w-0 flex-1 text-caption text-text-3">{m.phone_supervisor_note()}</span>
		{#if onresume}<Button label={m.supervisor_resume()} variant="secondary" iconEnd={Play} onclick={onresume} />{/if}
	</footer>
</section>
