<!-- frontend/src/lib/components/blocks/ApprovalCard.svelte -->
<script lang="ts">
	import { deserialize } from '$app/forms';
	import Checkbox from '$lib/components/m3/Checkbox.svelte';
	import type { ContentBlock } from '$lib/types';

	let {
		block,
		messageId,
	}: { block: Extract<ContentBlock, { kind: 'approval_request' }>; messageId: string } = $props();

	let remember = $state(false);
	let submitting = $state(false);
	let error = $state<string | null>(null);

	async function resolve(decision: 'approve' | 'deny') {
		submitting = true;
		error = null;
		const body = new FormData();
		body.set('messageId', messageId);
		body.set('decision', decision);
		body.set('remember', remember ? 'true' : 'false');

		const response = await fetch('?/resolveApproval', { method: 'POST', body });
		const result = deserialize(await response.text());
		submitting = false;
		if (result.type === 'failure') {
			error = (result.data?.error as string) ?? 'Failed to record your decision.';
		}
		// On success the card's own status flips via the MessageUpdated WS event landing shortly
		// after (Task 12) — no local optimistic update needed here.
	}
</script>

<div class="m3-block-card m3-block-card--approval" class:m3-block-card--settled={block.status !== 'pending'}>
	{#if block.status === 'pending'}
		<span class="nomi-meta m3-approval-eyebrow">Needs your OK</span>
	{/if}
	<p class="m3-approval-title">{block.description}</p>
	<p class="m3-approval-tool">Tool: <code>{block.tool_name}</code></p>

	{#if block.status === 'pending'}
		{#if error}
			<p class="md-body-small" style="color: var(--md-sys-color-error)">{error}</p>
		{/if}
		<label class="m3-approval-remember">
			<Checkbox bind:checked={remember} />
			<span class="md-body-small">Remember this decision for next time</span>
		</label>
		<div class="m3-block-card__actions">
			<button type="button" class="m3-approval-btn m3-approval-btn--allow" disabled={submitting} onclick={() => resolve('approve')}>
				Approve
			</button>
			<button type="button" class="m3-approval-btn m3-approval-btn--deny" disabled={submitting} onclick={() => resolve('deny')}>
				Deny
			</button>
		</div>
	{:else}
		<p class="m3-approval-result">
			{#if block.status === 'approved'}
				<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m5 12 5 5 9-10" /></svg>
				Approved
			{:else}
				<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" aria-hidden="true"><path d="M6 6l12 12M18 6 6 18" /></svg>
				Denied
			{/if}
		</p>
	{/if}
</div>

<style>
	/* Ember (tertiary) — the one place the warm accent appears in a conversation, so a
	   pending decision is impossible to scroll past. Settles to a neutral surface once decided. */
	.m3-block-card--approval {
		padding: 20px 22px;
		border-radius: var(--nomi-shape-bubble-start);
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.m3-block-card--settled {
		background: var(--md-sys-color-surface-container);
		color: var(--md-sys-color-on-surface);
	}
	.m3-approval-eyebrow {
		color: inherit;
		opacity: 0.8;
	}
	.m3-approval-title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.25rem;
		line-height: 1.3;
		font-weight: 650;
		letter-spacing: -0.01em;
	}
	.m3-approval-tool {
		margin: 0;
		font-size: var(--md-sys-typescale-body-small-size);
		opacity: 0.8;
	}
	.m3-approval-tool code {
		font-family: var(--md-ref-typeface-mono);
	}
	.m3-approval-result {
		display: flex;
		align-items: center;
		gap: 8px;
		margin: 0;
		font-weight: 650;
	}
	.m3-approval-btn {
		height: 48px;
		padding: 0 24px;
		border-radius: var(--md-sys-shape-corner-full);
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: 0.9375rem;
		font-weight: 600;
		cursor: pointer;
		transition: border-radius var(--nomi-motion-spatial-fast);
	}
	.m3-approval-btn:not(:disabled):active {
		border-radius: var(--md-sys-shape-corner-medium);
	}
	.m3-approval-btn:disabled {
		opacity: 0.38;
		cursor: not-allowed;
	}
	.m3-approval-btn--allow {
		border: none;
		background: var(--md-sys-color-tertiary);
		color: var(--md-sys-color-on-tertiary);
	}
	.m3-approval-btn--deny {
		border: 1.5px solid currentColor;
		background: transparent;
		color: inherit;
	}
	.m3-approval-remember {
		display: flex;
		align-items: center;
		gap: 6px;
		cursor: pointer;
	}
	.m3-block-card__actions {
		display: flex;
		gap: 8px;
	}
</style>
