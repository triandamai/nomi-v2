<script lang="ts">
	// A run that stopped (a turn, a build), posted by the server with what to rerun: the error
	// bubble, whose Try again queues it again and takes the bubble away.
	import { m } from '$lib/paraglide/messages';
	import { deserialize } from '$app/forms';
	import ErrorBubble from './ErrorBubble.svelte';
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
		retrying = true;
		retryError = null;
		const body = new FormData();
		body.set('messageId', messageId);
		const response = await fetch('?/retryRun', { method: 'POST', body });
		const result = deserialize(await response.text());
		retrying = false;
		if (result.type === 'success') {
			onremove?.();
		} else {
			retryError = result.type === 'failure' ? String(result.data?.error ?? m.run_failed_retry_error()) : m.run_failed_retry_error();
		}
	}
</script>

<ErrorBubble text={text || m.chat_turn_error()} onretry={block.retry ? retry : undefined} {retrying} {retryError} />
