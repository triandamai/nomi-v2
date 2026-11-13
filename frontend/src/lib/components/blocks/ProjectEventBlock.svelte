<script lang="ts">
	// Something that happened to the project. A publish that went live is the live card (Build →
	// Done): the address to copy or open, and Share. Anything else is logged in mono with its time
	// (Conversation → Event row): "production · failed · build_image · 15:38". The message's own
	// text (and a failed publish's log) follows it.
	import { m } from '$lib/paraglide/messages';
	import BuildDone from '$lib/components/ds/build/BuildDone.svelte';
	import EventRow from '$lib/components/ds/conversation/EventRow.svelte';
	import type { ContentBlock } from '$lib/types';

	let { block, time = '' }: { block: Extract<ContentBlock, { kind: 'project_event' }>; time?: string } = $props();

	/** Where a publish's logged line holds the address: "production · published · shop.example.app". */
	const LINE_ADDRESS_PART = 2;
	const parts = $derived(block.text.split(' · '));
	// Older messages carry the address only in the line.
	const url = $derived.by(() => {
		if (block.event !== 'deploy' || block.ok === false) return null;
		if (block.url) return block.url;
		const host = parts[LINE_ADDRESS_PART];
		return host && parts[1] === 'published' ? `https://${host}` : null;
	});
	const environment = $derived(block.environment ?? parts[0] ?? '');
</script>

{#if url}
	<BuildDone
		meta={[environment, time].filter(Boolean).join(' · ')}
		{url}
		summary={m.event_published_to({ env: environment })}
		onshare={() => window.dispatchEvent(new CustomEvent('nomi:open-sheet', { detail: 'members' }))}
	/>
{:else}
	<EventRow kind={block.event} text={block.text} {time} />
{/if}
