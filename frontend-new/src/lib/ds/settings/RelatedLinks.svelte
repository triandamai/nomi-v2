<script lang="ts">
	import type { Component } from 'svelte';
	import { ArrowRight, type LucideProps } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// App / Settings / General (m7lvY1) → Related: other places that touch these settings, under a
	// stroke: each its glyph, its name and an arrow at the end, 28 tall.
	type Props = { label: string; links: { href: string; label: string; icon: Component<LucideProps> }[] };
	let { label, links }: Props = $props();
</script>

<nav aria-label={label} class="flex flex-col gap-0.5 border-t border-border pt-3.75">
	<p class="pb-1.5 font-mono text-label tracking-label text-text-3 uppercase">{label}</p>
	{#each links as link (link.href)}
		<a href={link.href} class="group flex h-7 items-center gap-2 rounded-xs text-ui text-text-2 outline-offset-2 hover:text-text">
			<span class="inline-flex text-text-3"><Icon icon={link.icon} size={14} /></span>
			<span class="min-w-0 flex-1 truncate">{link.label}</span>
			<span class="inline-flex text-text-3 group-hover:text-text-2"><Icon icon={ArrowRight} size={12} /></span>
		</a>
	{/each}
</nav>
