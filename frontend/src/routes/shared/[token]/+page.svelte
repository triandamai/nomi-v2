<script lang="ts">
	// A shared reply or chat: read-only, no composer, no actions.
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import AgentByline from '$lib/components/ds/identity/AgentByline.svelte';
	import Button from '$lib/components/ds/actions/Button.svelte';
	import { crewId } from '$lib/crewId';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const time = (iso: string) => new Date(iso).toLocaleString(getLocale(), { day: 'numeric', month: 'short', hour: '2-digit', minute: '2-digit' });
</script>

<svelte:head><title>{m.shared_title()}</title><meta name="robots" content="noindex" /></svelte:head>

<main class="shared">
	{#if data.state === 'ready'}
		<header class="shared__head">
			<p class="nomi-meta">{m.shared_title()}</p>
			{#if data.chat.title}<h1 class="md-headline-medium">{data.chat.title}</h1>{/if}
			{#if data.chat.masked}<p class="shared__note">{m.shared_masked()}</p>{/if}
		</header>
		<ol class="shared__messages">
			{#each data.chat.messages as message, index (index)}
				<li class="shared__message" class:shared__message--user={message.sender === 'user'}>
					{#if message.sender === 'assistant'}
						<AgentByline agent={crewId(message.agent_display_name)} name={message.agent_display_name ?? 'Nomi'} meta={time(message.created_at)} />
					{/if}
					<div class="shared__text message-markdown">{@html message.html}</div>
				</li>
			{/each}
		</ol>
		{#if data.chat.expires_at}<p class="shared__note">{m.shared_expires({ when: time(data.chat.expires_at) })}</p>{/if}
	{:else if data.state === 'sign_in'}
		<p class="shared__note">{m.shared_sign_in()}</p>
		<Button label={m.shared_sign_in_action()} href="/login?redirect_to={encodeURIComponent(page.url.pathname)}" />
	{:else}
		<p class="shared__note">{m.shared_gone()}</p>
	{/if}
</main>

<style>
	:global(body) {
		background: var(--nomi-color-ground);
	}
	.shared {
		display: flex;
		max-width: 720px;
		flex-direction: column;
		gap: 24px;
		margin: 0 auto;
		padding: 48px 16px;
		color: var(--md-sys-color-on-surface);
	}
	.shared__head {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.shared__head h1,
	.shared__note {
		margin: 0;
	}
	.shared__note {
		color: var(--md-sys-color-on-surface-variant);
		font-size: var(--md-sys-typescale-body-medium-size);
	}
	.shared__messages {
		display: flex;
		flex-direction: column;
		gap: 20px;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.shared__message {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.shared__message--user {
		align-items: flex-end;
	}
	.shared__message--user .shared__text {
		max-width: 80%;
		padding: 10px 14px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.shared__text {
		font-size: var(--md-sys-typescale-body-large-size);
		line-height: var(--md-sys-typescale-body-large-line-height);
	}
</style>
