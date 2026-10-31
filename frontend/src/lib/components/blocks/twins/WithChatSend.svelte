<script lang="ts">
	// Test harness: gives a block the chat's send, as the chat page does.
	import type { Component } from 'svelte';
	import { provideChatSend, type ChatSend } from '$lib/chatSend';

	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	let { send, block: Block, props }: { send: ChatSend; block: Component<any>; props: Record<string, unknown> } = $props();

	provideChatSend((text) => send(text));
</script>

<Block {...props} />
