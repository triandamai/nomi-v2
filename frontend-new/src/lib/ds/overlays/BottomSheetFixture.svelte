<script lang="ts">
	import { Pin, Trash2 } from '@lucide/svelte';
	import BottomSheet from './BottomSheet.svelte';
	import SheetRow from '../mobile/SheetRow.svelte';

	// For the tests: the Chat sheet, open, with two of its rows.
	let { onpick }: { onpick?: (what: string) => void } = $props();
	let open = $state(true);
</script>

<button type="button" onclick={() => (open = true)}>Open</button>
<BottomSheet bind:open title="Food spending in October" subtitle="Dana · 14:02">
	<SheetRow icon={Pin} label="Pin to top" onclick={() => onpick?.('pin')} />
	<SheetRow icon={Trash2} label="Delete chat" danger onclick={() => onpick?.('delete')} />
</BottomSheet>
