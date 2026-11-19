<script lang="ts">
	import { Copy } from '@lucide/svelte';
	import BottomSheet from '../overlays/BottomSheet.svelte';
	import SheetRow from '../mobile/SheetRow.svelte';
	import PhoneMessageBubble from './PhoneMessageBubble.svelte';

	// For the tests: your message's sheet, open, with the bubble lifted over the scrim (nzT7f).
	let open = $state(true);
</script>

<BottomSheet bind:open title="Your message" subtitle="14:02">
	{#snippet lifted()}<PhoneMessageBubble text="How much did I spend on food in October? @Dana" lifted />{/snippet}
	<SheetRow icon={Copy} label="Copy text" onclick={() => {}} />
</BottomSheet>
