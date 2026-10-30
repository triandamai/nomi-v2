<script lang="ts">
	// What the person might want next, offered when the crew finished (FollowUps): picking one sends
	// it as their next message. Gone once they've said something else.
	import FollowUps from '$lib/components/ds/chat/FollowUps.svelte';
	import { useChatSend } from '$lib/chatSend';
	import type { ContentBlock } from '$lib/types';

	let { block, answered = false }: { block: Extract<ContentBlock, { kind: 'suggestions' }>; answered?: boolean } = $props();

	const send = useChatSend();
</script>

{#if !answered && send && block.items.length > 0}
	<FollowUps items={block.items} onpick={send} />
{/if}
