<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';

	// Mobile / FAB: a phone screen's one floating action, a 56 mint circle with its icon in
	// accent-ink and its own shadow (0 6 16): New chat on Chats (the pen's square-pen at 22), New
	// project on Projects (plus at 24). Whoever places it sets where: the pen floats it 16 from the
	// right, just above the tab bar. A link where it goes somewhere, a button where it opens a sheet.
	// The glyph is drawn as Icon draws one (1.5 stroke, decorative), at the 24 Icon doesn't offer.
	type Props = { label: string; icon: Component<LucideProps>; iconSize?: 22 | 24; href?: string; onclick?: () => void };
	let { label, icon: Glyph, iconSize = 24, href, onclick }: Props = $props();
</script>

{#if href}
	<a {href} aria-label={label} class="inline-flex size-14 items-center justify-center rounded-full bg-accent text-accent-ink shadow-fab outline-offset-2">
		<Glyph size={iconSize} strokeWidth={1.5} aria-hidden="true" />
	</a>
{:else}
	<button type="button" aria-label={label} class="inline-flex size-14 cursor-pointer items-center justify-center rounded-full bg-accent text-accent-ink shadow-fab outline-offset-2" {onclick}>
		<Glyph size={iconSize} strokeWidth={1.5} aria-hidden="true" />
	</button>
{/if}
