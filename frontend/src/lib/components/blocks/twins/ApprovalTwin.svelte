<script lang="ts">
	// A tool call waiting for the person's OK (ApprovalCard): Approve, optionally for good, or Deny.
	// The card's status flips when the MessageUpdated event lands; nothing changes here first.
	import { m } from '$lib/paraglide/messages';
	import { deserialize } from '$app/forms';
	import ApprovalCard from '$lib/components/ds/chat/ApprovalCard.svelte';
	import { crewId } from '$lib/crewId';
	import type { ContentBlock } from '$lib/types';

	const CONFLICT = 409;

	let {
		block,
		messageId,
		agent = null,
	}: { block: Extract<ContentBlock, { kind: 'approval_request' }>; messageId: string; agent?: string | null } = $props();

	let submitting = $state(false);
	let error = $state<string | null>(null);
	let expired = $state(false);

	const status = $derived(block.status === 'pending' ? 'pending' : block.status === 'approved' ? 'approved' : 'rejected');
	const outcome = $derived(
		block.status === 'approved' ? m.approval_approved() : block.status === 'cancelled' ? m.approval_stopped() : block.status === 'denied' ? m.approval_denied() : undefined,
	);
	const open = $derived(block.status === 'pending' && !expired && !submitting);

	async function resolve(decision: 'approve' | 'deny', remember = false) {
		submitting = true;
		error = null;
		const body = new FormData();
		body.set('messageId', messageId);
		body.set('decision', decision);
		body.set('remember', remember ? 'true' : 'false');
		const result = deserialize(await (await fetch('?/resolveApproval', { method: 'POST', body })).text());
		submitting = false;
		if (result.type === 'failure') {
			// 409: the turn behind this card has already moved on; retrying can't help, so say what to
			// do instead.
			expired = result.status === CONFLICT;
			error = expired ? m.approval_expired() : ((result.data?.error as string) ?? m.err_record_decision());
		}
	}
</script>

<ApprovalCard
	agent={crewId(agent)}
	title={block.description}
	{status}
	fields={[{ key: m.approval_tool(), value: block.tool_name }]}
	rememberLabel={m.approval_remember()}
	approveLabel={m.approval_approve()}
	rejectLabel={m.approval_deny()}
	{outcome}
	onapprove={open ? ({ remember }) => resolve('approve', remember) : undefined}
	onreject={open ? () => resolve('deny') : undefined}
/>
{#if error}<p class="error" role="alert">{error}</p>{/if}

<style>
	.error {
		margin: 6px 0 0;
		color: var(--md-sys-color-error);
		font-size: var(--md-sys-typescale-body-small-size);
	}
</style>
