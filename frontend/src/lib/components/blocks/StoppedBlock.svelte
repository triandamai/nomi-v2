<script lang="ts">
	// The Supervisor's reply to "stop" (Chat responses → Supervisor Stopped): what it said, then
	// what was stopped, task by task: stopped part-way, or never started. Resume asks the crew to
	// pick it back up, through the chat's composer.
	import { m } from '$lib/paraglide/messages';
	import { useChatSend } from '$lib/chatSend';
	import { crewId } from '$lib/crewId';
	import SupervisorStopped from '$lib/components/ds/chat/SupervisorStopped.svelte';
	import type { ContentBlock } from '$lib/types';

	let {
		block,
		text = '',
		answered = false,
	}: { block: Extract<ContentBlock, { kind: 'stopped' }>; text?: string; answered?: boolean } = $props();

	const send = useChatSend();
	const tasks = $derived(block.tasks.map((task) => ({ agent: crewId(task.agent), title: task.title, detail: task.detail ?? undefined, state: task.state })));
</script>

<div class="stopped">
	{#if text}<p class="stopped__text">{text}</p>{/if}
	<SupervisorStopped {tasks} onresume={send && !answered ? () => send(m.supervisor_resume_message()) : undefined} />
</div>

<style>
	.stopped {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.stopped__text {
		margin: 0;
	}
</style>
