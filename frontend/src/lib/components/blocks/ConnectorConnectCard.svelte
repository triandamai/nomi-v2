<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	// The crew needs an app the person hasn't connected (or whose key stopped working): a card
	// that opens its setup page, with this chat to come back to.
	import { page } from '$app/state';
	import Button from '$lib/components/m3/Button.svelte';
	import { CONNECTORS } from '$lib/connectors';
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'connector_connect' }> } = $props();

	let dismissed = $state(false);
	const name = $derived(CONNECTORS.find((c) => c.id === block.connector)?.name ?? block.connector);
	const rejected = $derived(block.reason === 'key_rejected');
	const href = $derived.by(() => {
		const params = new URLSearchParams();
		if (page.params.sessionId) params.set('resume', page.params.sessionId);
		if (rejected) params.set('reason', 'key_rejected');
		const query = params.toString();
		return `/connections/${block.connector}${query ? `?${query}` : ''}`;
	});
</script>

{#if !dismissed}
	<div class="connect">
		<p class="connect__title">{rejected ? m.cc_key_stopped({ name }) : m.cc_connect_title({ name })}</p>
		<p class="connect__body">{rejected ? m.cc_key_stopped_body({ name }) : m.cc_connect_body({ name })}</p>
		<div class="connect__actions">
			<Button variant="filled" size="s" {href}>{rejected ? m.cc_update_key() : m.cc_connect({ name })}</Button>
			<Button variant="text" size="s" onclick={() => (dismissed = true)}>{m.common_not_now()}</Button>
		</div>
	</div>
{/if}

<style>
	.connect {
		display: flex;
		flex-direction: column;
		gap: 10px;
		max-width: 420px;
		padding: 16px 16px;
		border-radius: var(--nomi-shape-bubble-start);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.connect__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.0625rem;
		font-weight: 650;
		line-height: 1.35;
	}
	.connect__body {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		line-height: 1.5;
	}
	.connect__actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		align-items: center;
	}
	.connect__actions :global(a) {
		text-decoration: none;
	}
</style>
