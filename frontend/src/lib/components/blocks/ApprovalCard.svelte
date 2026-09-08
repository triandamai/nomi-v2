<!-- frontend/src/lib/components/blocks/ApprovalCard.svelte -->
<script lang="ts">
	import { deserialize } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
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

<div class="m3-block-card m3-block-card--approval">
	<p class="md-title-medium" style="color: var(--md-sys-color-on-surface)">{block.description}</p>
	<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">Tool: <code>{block.tool_name}</code></p>

	{#if block.status === 'pending'}
		{#if error}
			<p class="md-body-small" style="color: var(--md-sys-color-error)">{error}</p>
		{/if}
		<label class="m3-approval-remember">
			<Checkbox bind:checked={remember} />
			<span class="md-body-small">Remember this decision for next time</span>
		</label>
		<div class="m3-block-card__actions">
			<Button type="button" variant="filled" disabled={submitting} onclick={() => resolve('approve')}>Approve</Button>
			<Button type="button" variant="outlined" disabled={submitting} onclick={() => resolve('deny')}>Deny</Button>
		</div>
	{:else}
		<p
			class="md-label-large"
			style="color: {block.status === 'approved' ? 'var(--md-sys-color-primary)' : 'var(--md-sys-color-error)'}"
		>
			{block.status === 'approved' ? '✓ Approved' : '✗ Denied'}
		</p>
	{/if}
</div>

<style>
	.m3-block-card--approval {
		padding: 14px;
		border-radius: var(--md-sys-shape-corner-medium);
		border: 1px solid var(--md-sys-color-outline-variant);
		background: var(--md-sys-color-surface-container-low);
		display: flex;
		flex-direction: column;
		gap: 8px;
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
