<!-- Short answers to the question just asked (Chat responses → Quick Replies), one tap each; gone
     once the person has answered. -->
<script lang="ts">
	import { useChatSend } from '$lib/chatSend';
	import QuickReplies from '$lib/components/ds/chat/QuickReplies.svelte';
	import type { QuickRepliesView, ViewProps } from '$lib/views/types';

	let { view, answered = false }: ViewProps<QuickRepliesView> = $props();

	const send = useChatSend();
	let sent = $state(false);
</script>

{#if !answered && !sent && send}
	<QuickReplies
		replies={view.replies}
		onpick={(reply) => {
			sent = true;
			send(reply);
		}}
	/>
{/if}
