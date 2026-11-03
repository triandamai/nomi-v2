<!-- Work going on in the background: what it is, who has it, where it stands, how far along, and
     that the result comes back here and to Inbox. -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import WavyProgress from '$lib/components/m3/WavyProgress.svelte';
	import ViewFrame from './ViewFrame.svelte';
	import BackgroundNote from '$lib/components/ds/chat/BackgroundNote.svelte';
	import KVRow from '$lib/components/ds/conversation/KVRow.svelte';
	import { Terminal } from '@lucide/svelte';
	import type { TaskView, ViewProps } from '$lib/views/types';

	let { view, agent = null }: ViewProps<TaskView> = $props();

	const status = $derived(view.status ?? 'queued');
	const STATUS = { queued: m.view_task_queued, running: m.view_task_running, done: m.view_task_done, failed: m.view_task_failed, cancelled: m.view_task_cancelled };
	const who = $derived(view.agent ?? agent);
</script>

<ViewFrame>
	<div class="task task--{status}">
		<div class="task__head">
			<AgentShape agent={who} size={32} face working={status === 'running'} />
			<span class="task__text">
				<span class="task__title">{view.title}</span>
				{#if view.detail && !view.id}<span class="task__detail">{view.detail}</span>{/if}
			</span>
			<span class="task__status">{STATUS[status]()}</span>
		</div>
		<!-- A background task the server started: its command, as a machine value (KVRow). -->
		{#if view.detail && view.id}<KVRow icon={Terminal} label={m.view_task_command()} value={view.detail} />{/if}
		{#if view.progress !== undefined && view.progress !== null && status !== 'failed' && status !== 'cancelled'}
			<WavyProgress value={status === 'done' ? 1 : view.progress} label={view.title} />
		{/if}
	</div>
</ViewFrame>
<!-- Work still going on runs without the person watching (Chat responses → Background Note). -->
{#if status === 'queued' || status === 'running'}<BackgroundNote eta={view.eta ? (view.eta.startsWith("~") ? view.eta : `~${view.eta}`) : undefined} />{/if}

<style>
	.task {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.task__head {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.task__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		flex: 1;
		min-width: 0;
	}
	.task__title {
		font-weight: 700;
		line-height: 1.35;
	}
	.task__detail {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		line-height: 1.4;
	}
	.task__status {
		flex: none;
		padding: 4px 10px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		font-weight: 600;
	}
	.task--running .task__status {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.task--done .task__status {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.task--failed .task__status {
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}
</style>
