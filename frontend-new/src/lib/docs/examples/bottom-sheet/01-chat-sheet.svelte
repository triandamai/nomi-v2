<!-- Chat sheet: a chat's actions on a phone, opened from its More. -->
<script lang="ts">
	import { Download, FolderInput, Pencil, Pin, Share, Trash2 } from '@lucide/svelte';
	import BottomSheet from '$lib/ds/overlays/BottomSheet.svelte';
	import SheetRow from '$lib/ds/mobile/SheetRow.svelte';
	import Button from '$lib/ds/actions/Button.svelte';

	let open = $state(false);
	const close = () => (open = false);
</script>

<Button variant="secondary" label="Open the chat sheet" onclick={() => (open = true)} />
<BottomSheet bind:open title="Food spending in October" subtitle="Dana · 14:02">
	<SheetRow icon={Pencil} label="Rename" onclick={close} />
	<SheetRow icon={Pin} label="Pin to top" onclick={close} />
	<SheetRow icon={FolderInput} label="Move to a project" more onclick={close} />
	<SheetRow icon={Download} label="Export" meta="md · pdf" onclick={close} />
	<SheetRow icon={Share} label="Share whole chat" onclick={close} />
	<SheetRow icon={Trash2} label="Delete chat" danger onclick={close} />
</BottomSheet>
