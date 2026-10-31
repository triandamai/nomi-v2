<script lang="ts">
	// A run that stopped (a turn, a build), posted by the server with what to rerun (ErrorCallout):
	// Try again queues it again and takes the message away.
	import { RotateCcw } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { deserialize } from '$app/forms';
	import ErrorCallout from '$lib/components/ds/conversation/ErrorCallout.svelte';
	import type { ContentBlock } from '$lib/types';

	let {
		block,
		messageId,
		text = '',
		onremove,
	}: {
		block: Extract<ContentBlock, { kind: 'run_failed' }>;
		messageId: string;
		/** The message's text: what went wrong. */
		text?: string;
		onremove?: () => void;
	} = $props();

	let retrying = $state(false);
	let retryError = $state<string | null>(null);

	async function retry() {
		if (retrying) return;
		retrying = true;
		retryError = null;
		const body = new FormData();
		body.set('messageId', messageId);
		const result = deserialize(await (await fetch('?/retryRun', { method: 'POST', body })).text());
		retrying = false;
		if (result.type === 'success') onremove?.();
		else retryError = result.type === 'failure' ? String(result.data?.error ?? m.run_failed_retry_error()) : m.run_failed_retry_error();
	}
</script>

<ErrorCallout
	title={text || m.chat_turn_error()}
	actions={block.retry ? [{ label: retrying ? m.run_failed_retrying() : m.run_failed_retry(), icon: RotateCcw, onclick: retry }] : []}
/>
{#if retryError}<p class="retry-error" role="alert">{retryError}</p>{/if}

<style>
	.retry-error {
		margin: 6px 0 0;
		color: var(--md-sys-color-error);
		font-size: var(--md-sys-typescale-body-small-size);
	}
</style>
