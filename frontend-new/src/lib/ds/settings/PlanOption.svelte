<script lang="ts">
	import type { Snippet } from 'svelte';
	import { Check } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Tag from '../status/Tag.svelte';

	// App / Settings / Plan & usage (C0og49) → Plan: one plan to compare, outlined in border-strong,
	// 20 in: its name and tag (the one they're on in the accent), the price big in mono with what
	// it's per, what it's for, and what it includes under a stroke, each with a check. `raised`
	// puts it on the surface, as the pen sets the plan above theirs apart.
	type Props = {
		name: string;
		tag?: { label: string; kind: 'go' | 'neutral' };
		price: string;
		per?: string;
		description: string;
		features: string[];
		raised?: boolean;
		footer?: Snippet;
	};
	let { name, tag, price, per, description, features, raised = false, footer }: Props = $props();
</script>

<article aria-label={name} class={['flex min-w-0 flex-col gap-3.5 rounded-md p-5 outline -outline-offset-1 outline-border-strong', raised && 'bg-surface']}>
	<header class="flex items-center justify-between gap-2">
		<h3 class="text-body font-medium text-text">{name}</h3>
		{#if tag}<Tag kind={tag.kind} label={tag.label} />{/if}
	</header>
	<p class="flex items-baseline gap-1.5">
		<span class="font-mono text-display font-medium text-text">{price}</span>
		{#if per}<span class="font-mono text-label text-text-3">{per}</span>{/if}
	</p>
	<p class="text-ui text-text-2">{description}</p>
	{#if features.length}
		<ul class="flex flex-col gap-2 border-t border-border pt-3.25">
			{#each features as feature (feature)}
				<li class="flex items-start gap-2 text-ui text-text-2">
					<span class="mt-px inline-flex shrink-0"><Icon icon={Check} size={14} /></span>{feature}
				</li>
			{/each}
		</ul>
	{/if}
	{#if footer}<div class="flex items-center gap-2.5 pt-1">{@render footer()}</div>{/if}
</article>
