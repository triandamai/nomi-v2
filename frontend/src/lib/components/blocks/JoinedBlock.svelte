<script lang="ts">
	// Someone joined the project's chat (Build together → Joined note): what they can do, and
	// who publishes for them.
	import JoinedNote from '$lib/components/ds/build/JoinedNote.svelte';
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'member_joined' }> } = $props();
</script>

<JoinedNote name={block.name} role={block.role} publisher={block.publisher ?? undefined} />
