<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	// The Workspace agent needs the user's own Google account (or a service they haven't allowed):
	// a card that takes them to connect it, then back to this chat, where the request carries on.
	import { page } from '$app/state';
	import Button from '$lib/components/m3/Button.svelte';
	import { connectLink, serviceLabel } from '$lib/workspace';
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'workspace_connect' }> } = $props();

	let dismissed = $state(false);
	const missingService = $derived(block.reason === 'service_not_allowed');
	const services = $derived(block.services.map(serviceLabel).join(', '));
	const href = $derived(connectLink(block.services, page.params.sessionId ?? null));
</script>

{#if !dismissed}
	<div class="connect">
		<p class="connect__title">
			{missingService ? m.wsc_turn_on_for({ services }) : m.wsc_need_account()}
		</p>
		<p class="connect__body">
			{#if missingService}
				{m.wsc_missing_body({ services })}
			{:else}
				{m.wsc_connect_body()}
			{/if}
		</p>
		<div class="connect__actions">
			<Button variant="filled" size="s" {href}>{missingService ? m.wsc_turn_on({ services }) : m.wsc_connect()}</Button>
			<Button variant="text" size="s" onclick={() => (dismissed = true)}>{m.common_not_now()}</Button>
		</div>
		<p class="connect__note">{m.wsc_note()}</p>
	</div>
{/if}

<style>
	.connect {
		display: flex;
		flex-direction: column;
		gap: 10px;
		max-width: 420px;
		padding: 18px 20px;
		border-radius: var(--nomi-shape-bubble-start);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.connect__title {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.125rem;
		font-weight: 650;
		line-height: 1.35;
	}
	.connect__body {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		line-height: 1.5;
	}
	.connect__actions :global(a) {
		text-decoration: none;
	}
	.connect__actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		align-items: center;
	}
	.connect__note {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.75rem;
		line-height: 1.45;
	}
</style>
