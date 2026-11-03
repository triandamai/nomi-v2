<script lang="ts">
	// Something that happened to the project, logged in mono with its time (Design System / Nomi →
	// Conversation → Event row): "production · published · shop.example.app · 15:38". The
	// message's own text (and a failed publish's log) follows it.
	import EventRow from '$lib/components/ds/conversation/EventRow.svelte';
	import type { ContentBlock } from '$lib/types';

	let { block, time = '' }: { block: Extract<ContentBlock, { kind: 'project_event' }>; time?: string } = $props();
</script>

<EventRow kind={block.event} text={block.text} {time} />
