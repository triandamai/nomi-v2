<script lang="ts">
	import { ArrowUpRight, ChevronRight, Shield } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Chat responses → Boundary: something the crew won't do, said plainly with the reason, and
	// what it can do instead, each owned by the crew member who'd do it.
	type Alternative = { agent: CrewId; title: string; detail?: string; href?: string; external?: boolean; onclick?: () => void };
	type Props = { title: string; reason: string; alternatives?: Alternative[] };
	let { title, reason, alternatives = [] }: Props = $props();
</script>

{#snippet inside(alternative: Alternative)}
	<Mascot agent={alternative.agent} size={20} />
	<span class="flex min-w-0 flex-1 flex-col gap-px">
		<span class="text-ui text-text">{alternative.title}</span>
		{#if alternative.detail}
			<span class="inline-flex items-center gap-1 font-mono text-label text-text-3">
				{alternative.detail}{#if alternative.external}<Icon icon={ArrowUpRight} size={12} />{/if}
			</span>
		{/if}
	</span>
	<span class="inline-flex shrink-0 text-text-3"><Icon icon={ChevronRight} size={12} /></span>
{/snippet}

<section class="flex w-full flex-col overflow-hidden rounded-md bg-bg outline -outline-offset-1 outline-border-strong">
	<header class="flex items-center gap-2.5 border-b border-border bg-surface px-3.5 pt-2.5 pb-2.25">
		<span class="inline-flex text-text-2"><Icon icon={Shield} size={14} /></span>
		<p class="min-w-0 flex-1 text-ui font-medium text-text">{title}</p>
	</header>
	<div class="flex flex-col gap-2.5 p-3.5">
		<p class="text-ui leading-normal text-text-2">{reason}</p>
		{#each alternatives as alternative (alternative.title)}
			{#if alternative.href}
				<a
					href={alternative.href}
					target={alternative.external ? '_blank' : undefined}
					rel={alternative.external ? 'noopener noreferrer' : undefined}
					class="flex items-center gap-2.5 rounded-sm border border-border-strong px-2.25 py-1.75 outline-offset-2 transition-colors hover:bg-surface">{@render inside(alternative)}</a
				>
			{:else}
				<button type="button" class="flex cursor-pointer items-center gap-2.5 rounded-sm border border-border-strong px-2.25 py-1.75 text-left outline-offset-2 transition-colors hover:bg-surface" onclick={alternative.onclick}>
					{@render inside(alternative)}
				</button>
			{/if}
		{/each}
	</div>
</section>
