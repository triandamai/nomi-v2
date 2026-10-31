<script lang="ts">
	import { BatteryFull, Signal, Wifi } from '@lucide/svelte';

	// The phone's own status bar (9:41, signal, wifi, battery), drawn around the app where the
	// design shows a phone: decoration, hidden from assistive tech.
</script>

<div aria-hidden="true" class="flex h-13.5 shrink-0 items-center justify-between pr-7 pl-10 text-text">
	<span class="text-phone-title font-semibold">9:41</span>
	<span class="flex items-center gap-1.5">
		<Signal size={17} strokeWidth={1.5} /><Wifi size={17} strokeWidth={1.5} /><BatteryFull size={25} strokeWidth={1.5} />
	</span>
</div>
