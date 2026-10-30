<script lang="ts">
	// A reminder going off in chat, posted by the Reminders agent (Reminder): Done, or Snooze.
	import { untrack } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import Reminder from '$lib/components/ds/chat/Reminder.svelte';
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'reminder' }> } = $props();

	const SNOOZE_MINUTES = 60;
	const MINUTE_MS = 60_000;

	// Starts where the reminder stands now (the chat adds `state` when it loads), so a card
	// answered before a reload doesn't offer Done and Snooze again.
	type Status = 'open' | 'done' | 'snoozed' | 'cancelled' | 'gone' | 'error';
	let status = $state<Status>(untrack(() => block.state ?? 'open'));
	let snoozedUntil = $state<string | null>(untrack(() => block.snoozed_until ?? null));
	let busy = $state(false);

	const time = (iso: string) => new Date(iso).toLocaleString(getLocale(), { weekday: 'short', hour: '2-digit', minute: '2-digit' });

	async function act(action: 'done' | 'snooze') {
		if (busy) return;
		busy = true;
		try {
			const minutes = action === 'snooze' ? SNOOZE_MINUTES : undefined;
			const response = await fetch(`/reminders/${block.reminder_id}`, {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({ action, minutes }),
			});
			if (response.ok) {
				status = action === 'done' ? 'done' : 'snoozed';
				snoozedUntil = action === 'snooze' ? new Date(Date.now() + SNOOZE_MINUTES * MINUTE_MS).toISOString() : null;
			} else {
				// Already handled elsewhere (or deleted): show that, not a dead pair of buttons.
				status = response.status === 404 ? 'gone' : 'error';
			}
		} catch {
			status = 'error';
		} finally {
			busy = false;
		}
	}
</script>

<Reminder
	title={block.title}
	when={time(block.due_at)}
	due={status === 'open' || status === 'error'}
	owner={{ agent: 'kala', name: 'Kala' }}
	ondone={status === 'open' || status === 'error' ? () => act('done') : undefined}
	onsnooze={status === 'open' || status === 'error' ? () => act('snooze') : undefined}
/>
{#if status !== 'open'}
	<p class="result" role={status === 'error' ? 'alert' : 'status'}>
		{#if status === 'done'}{m.remblock_done()}
		{:else if status === 'snoozed'}{snoozedUntil ? m.remblock_snoozed_until({ when: time(snoozedUntil) }) : m.remblock_snoozed()}
		{:else if status === 'cancelled'}{m.remblock_cancelled()}
		{:else if status === 'error'}{m.remblock_error()}
		{:else}{m.remblock_gone()}{/if}
	</p>
{/if}

<style>
	.result {
		margin: 6px 0 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: var(--md-sys-typescale-body-small-size);
	}
	.result[role='alert'] {
		color: var(--md-sys-color-error);
	}
</style>
