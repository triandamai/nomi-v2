<script lang="ts">
	import type { Component } from 'svelte';
	import type { LucideProps } from '@lucide/svelte';

	// Lucide only, at the design's 1.5 stroke: 16px in nav and buttons, 14px inline with text,
	// 12px in chips. Decorative unless it has a label (an icon that means something on its own).
	let { icon: Glyph, size = 16, label }: { icon: Component<LucideProps>; size?: 16 | 14 | 12; label?: string } = $props();
</script>

{#if label}
	<Glyph {size} strokeWidth={1.5} role="img" aria-label={label} />
{:else}
	<Glyph {size} strokeWidth={1.5} />
{/if}
