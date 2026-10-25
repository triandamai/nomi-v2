<script lang="ts">
	// What the person might want next, offered when the crew finished: tapping one sends it as
	// their next message. Gone once they've said something else.
	import { m } from '$lib/paraglide/messages';
	import Chip from '$lib/components/m3/Chip.svelte';
	import { useChatSend } from '$lib/chatSend';
	import type { ContentBlock } from '$lib/types';

	let { block, answered = false }: { block: Extract<ContentBlock, { kind: 'suggestions' }>; answered?: boolean } = $props();

	const send = useChatSend();
</script>

{#if !answered && send && block.items.length > 0}
	<div class="suggestions" role="group" aria-label={m.suggestions_label()}>
		{#each block.items as item (item)}
			<Chip variant="suggestion" onclick={() => send(item)}>{item}</Chip>
		{/each}
	</div>
{/if}

<style>
	.suggestions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
</style>
