<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';

	// Lucide only, at the design's 1.5 stroke: 16px in nav and buttons, 14px inline with text,
	// 12px in chips; other sizes only where the pen draws them (13, 18, the phone's 15 and 20–26).
	// Decorative unless it has a label (an icon that means something on its own).
	let { icon: Glyph, size = 16, label }: { icon: Component<LucideProps>; size?: 26 | 22 | 20 | 18 | 16 | 15 | 14 | 13 | 12; label?: string } = $props();
</script>

{#if label}
	<Glyph {size} strokeWidth={1.5} role="img" aria-label={label} />
{:else}
	<Glyph {size} strokeWidth={1.5} />
{/if}
