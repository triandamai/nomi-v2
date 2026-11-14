<!-- What the app uses. -->
<script lang="ts">
	import { ArrowDownUp, Cpu, MemoryStick } from '@lucide/svelte';
	import ResourceRow from '$lib/ds/projects/ResourceRow.svelte';
</script>

<div class="flex w-full max-w-170 flex-col">
	<ResourceRow icon={MemoryStick} name="Memory" value="61 / 256 MB" />
	<ResourceRow icon={Cpu} name="CPU" value="4%" />
	<ResourceRow icon={ArrowDownUp} name="Bandwidth" value="18 MB" />
</div>
