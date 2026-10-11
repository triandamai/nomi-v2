<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import type { ContentBlock } from '$lib/types';
	import { untrack } from 'svelte';

	// A reminder going off in chat (posted by the Reminders agent): Done, or Snooze.

	let { block }: { block: Extract<ContentBlock, { kind: 'reminder' }> } = $props();

	// Starts where the reminder stands now (the chat adds `state` when it loads), so a card
	// answered before a reload doesn't offer Done and Snooze again.
	type Status = 'open' | 'done' | 'snoozed' | 'cancelled' | 'gone' | 'error';
	let status = $state<Status>(untrack(() => block.state ?? 'open'));
	let snoozedUntil = $state<string | null>(untrack(() => block.snoozed_until ?? null));
	let busy = $state(false);

	const time = (iso: string) => new Date(iso).toLocaleString(getLocale(), { weekday: 'short', hour: '2-digit', minute: '2-digit' });

	const when = $derived(
		new Date(block.due_at).toLocaleString(getLocale(), { weekday: 'short', hour: '2-digit', minute: '2-digit' }),
	);

	async function act(action: 'done' | 'snooze', minutes?: number) {
		busy = true;
		try {
			const response = await fetch(`/reminders/${block.reminder_id}`, {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({ action, minutes }),
			});
			if (response.ok) {
				status = action === 'done' ? 'done' : 'snoozed';
				snoozedUntil = action === 'snooze' ? new Date(Date.now() + (minutes ?? 10) * 60_000).toISOString() : null;
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

<section class="reminder" class:reminder--settled={status !== 'open' && status !== 'error'} aria-label={m.remblock_label({ title: block.title })}>
	<div class="reminder__head">
		<AgentShape agent="reminders" size={28} working={status === 'open'} face />
		<span class="nomi-meta reminder__eyebrow">{m.remblock_eyebrow({ when })}</span>
	</div>
	<p class="reminder__title">{block.title}</p>
	{#if block.notes}<p class="reminder__notes">{block.notes}</p>{/if}

	{#if status === 'open' || status === 'error'}
		<div class="reminder__actions">
			<Button variant="filled" disabled={busy} onclick={() => act('done')}>{m.common_done()}</Button>
			<Button variant="tonal" disabled={busy} onclick={() => act('snooze', 10)}>{m.remblock_snooze10()}</Button>
			<Button variant="text" disabled={busy} onclick={() => act('snooze', 60)}>{m.remblock_1h()}</Button>
		</div>
		{#if status === 'error'}
			<p class="reminder__error" role="alert">{m.remblock_error()}</p>
		{/if}
	{:else}
		<p class="reminder__result" role="status">
			{#if status === 'done'}{m.remblock_done()}
			{:else if status === 'snoozed'}{snoozedUntil ? m.remblock_snoozed_until({ when: time(snoozedUntil) }) : m.remblock_snoozed()}
			{:else if status === 'cancelled'}{m.remblock_cancelled()}
			{:else}{m.remblock_gone()}{/if}
		</p>
	{/if}
</section>

<style>
	/* Citrus: the Reminders agent's gradient, as a tinted container with the speech corner. */
	.reminder {
		display: flex;
		flex-direction: column;
		gap: 8px;
		max-width: 480px;
		padding: 18px 20px;
		border-radius: var(--nomi-shape-bubble-start);
		background: color-mix(in srgb, #ffc23c 22%, var(--md-sys-color-surface-container-lowest));
		color: var(--md-sys-color-on-surface);
		transition: background-color var(--nomi-motion-effects-default);
	}
	.reminder--settled {
		background: var(--md-sys-color-surface-container);
	}
	.reminder__head {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.reminder__eyebrow {
		color: inherit;
	}
	.reminder__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.375rem;
		font-weight: 700;
		line-height: 1.25;
	}
	.reminder__notes,
	.reminder__result {
		margin: 0;
		font-size: 0.9375rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.reminder__actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-top: 4px;
	}
	.reminder__error {
		margin: 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-error);
	}
</style>
