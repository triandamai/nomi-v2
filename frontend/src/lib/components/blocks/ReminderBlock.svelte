<script lang="ts">
	import { getLocale } from '$lib/paraglide/runtime';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import type { ContentBlock } from '$lib/types';

	// A reminder going off in chat (posted by the Reminders agent): Done, or Snooze.

	let { block }: { block: Extract<ContentBlock, { kind: 'reminder' }> } = $props();

	let status = $state<'open' | 'done' | 'snoozed' | 'error'>('open');
	let busy = $state(false);

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
			status = response.ok ? (action === 'done' ? 'done' : 'snoozed') : 'error';
		} catch {
			status = 'error';
		} finally {
			busy = false;
		}
	}
</script>

<section class="reminder" class:reminder--settled={status === 'done' || status === 'snoozed'} aria-label="Reminder: {block.title}">
	<div class="reminder__head">
		<AgentShape agent="reminders" size={28} working={status === 'open'} />
		<span class="nomi-meta reminder__eyebrow">Reminder · {when}</span>
	</div>
	<p class="reminder__title">{block.title}</p>
	{#if block.notes}<p class="reminder__notes">{block.notes}</p>{/if}

	{#if status === 'open' || status === 'error'}
		<div class="reminder__actions">
			<Button variant="filled" disabled={busy} onclick={() => act('done')}>Done</Button>
			<Button variant="tonal" disabled={busy} onclick={() => act('snooze', 10)}>Snooze 10 min</Button>
			<Button variant="text" disabled={busy} onclick={() => act('snooze', 60)}>1 hour</Button>
		</div>
		{#if status === 'error'}
			<p class="reminder__error" role="alert">That reminder can't be changed any more.</p>
		{/if}
	{:else}
		<p class="reminder__result" role="status">
			{status === 'done' ? 'Done. Nice one.' : 'Snoozed. It will come back shortly.'}
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
