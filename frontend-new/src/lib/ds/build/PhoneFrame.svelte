<script lang="ts">
	import type { Snippet } from 'svelte';
	import { BatteryFull, Signal, Wifi } from '@lucide/svelte';

	// Build → Phone preview: the app at real phone size, light, inside the dark workspace: a bezel,
	// the phone's status bar (decoration), and the app itself (an iframe of the preview, or
	// content for the gallery).
	type Props = { src?: string; title?: string; children?: Snippet };
	let { src, title, children }: Props = $props();
</script>

<div data-phone data-theme="light" class="flex h-164 w-86.5 shrink-0 flex-col overflow-hidden rounded-device border-8 border-bezel bg-bg text-text">
	<div aria-hidden="true" class="flex h-13.5 shrink-0 items-center justify-between pr-7 pl-10">
		<span class="text-phone-title font-semibold">9:41</span>
		<span class="flex items-center gap-1.5">
			<Signal size={17} strokeWidth={1.5} /><Wifi size={17} strokeWidth={1.5} /><BatteryFull size={25} strokeWidth={1.5} />
		</span>
	</div>
	{#if src}
		<iframe {src} {title} class="min-h-0 w-full flex-1 border-0"></iframe>
	{:else}
		<div class="flex min-h-0 flex-1 flex-col">{@render children?.()}</div>
	{/if}
</div>
