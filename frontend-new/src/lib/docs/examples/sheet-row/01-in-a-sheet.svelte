<!-- In a sheet: one action per row under the sheet's header; danger for what deletes. -->
<script lang="ts">
	import { RefreshCw, Trash2 } from '@lucide/svelte';
	import SheetHeader from '$lib/ds/mobile/SheetHeader.svelte';
	import SheetRow from '$lib/ds/mobile/SheetRow.svelte';
</script>

<div class="w-97.5 max-w-full">
	<SheetHeader title="Reply" subtitle="Dana · 14:02" onclose={() => {}} />
	<SheetRow icon={RefreshCw} label="Regenerate" onclick={() => {}} />
	<SheetRow icon={Trash2} label="Delete reply" danger onclick={() => {}} />
</div>
