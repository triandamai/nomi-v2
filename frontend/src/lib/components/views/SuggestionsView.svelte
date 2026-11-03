<!-- Next things the person might ask (Chat responses → Follow-ups): picking one sends it as their message. -->
<script lang="ts">
	import { useChatSend } from '$lib/chatSend';
	import FollowUps from '$lib/components/ds/chat/FollowUps.svelte';
	import type { SuggestionsView, ViewProps } from '$lib/views/types';

	let { view, answered = false }: ViewProps<SuggestionsView> = $props();

	const send = useChatSend();
	let sent = $state<string | null>(null);
	const done = $derived(answered || sent !== null);

	function pick(item: string) {
		if (done || !send) return;
		send(item);
		sent = item;
	}
</script>

<!-- Once the conversation has moved on, the follow-ups have done their job: they go. -->
{#if !answered && !sent && send && view.items.length > 0}
	<div class="suggestions" role="group" aria-label={view.title}>
		{#if view.title}<span class="suggestions__title">{view.title}</span>{/if}
		<FollowUps items={view.items} onpick={pick} />
	</div>
{/if}

<style>
	.suggestions {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.suggestions__title {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
</style>
