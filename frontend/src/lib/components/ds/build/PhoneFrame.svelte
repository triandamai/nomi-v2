<script lang="ts">
	import type { Snippet } from 'svelte';
	import PhoneStatusBar from './PhoneStatusBar.svelte';

	// Build → Phone preview: the app at real phone size (330 × 640), light, inside the dark workspace:
	// an 8px bezel drawn on its edge as the pen does (half over the app, half outside, no layout),
	// the phone's status bar (decoration), and the app itself (an iframe of the preview, or content
	// for the gallery).
	type Props = { src?: string; title?: string; children?: Snippet };
	let { src, title, children }: Props = $props();
</script>

<div data-phone data-theme="light" class="flex h-160 w-82.5 shrink-0 flex-col overflow-hidden rounded-device bg-bg text-text outline-8 -outline-offset-4 outline-bezel">
	<PhoneStatusBar />
	{#if src}
		<iframe {src} {title} class="min-h-0 w-full flex-1 border-0"></iframe>
	{:else}
		<div class="flex min-h-0 flex-1 flex-col">{@render children?.()}</div>
	{/if}
</div>
