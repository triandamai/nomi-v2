<script lang="ts">
	import { Brain, ChevronDown, ChevronRight, LoaderCircle } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Chat responses → Reasoning: live while the model thinks ("Thinking 00:04" over the stream),
	// then folded away once the answer lands ("Thought for 8s"), open on request.
	type Props = { live?: boolean; label: string; elapsed?: string; text: string; open?: boolean };
	let { live = false, label, elapsed, text, open = $bindable(false) }: Props = $props();

	const id = $props.id();
	const paragraphs = $derived(text.split(/\n\s*\n/).filter((part) => part.trim()));
</script>

{#snippet body()}
	<div {id} class="flex flex-col gap-2.5 border-l border-border-strong py-0.5 pl-3.5">
		{#each paragraphs as paragraph, index (index)}<p class="text-ui leading-relaxed text-text-3">{paragraph}</p>{/each}
	</div>
{/snippet}

<div class="flex w-full flex-col gap-2">
	{#if live}
		<span class="inline-flex items-center gap-1.5 text-ui text-text-2">
			<span class="inline-flex animate-spin motion-reduce:animate-none"><Icon icon={LoaderCircle} size={14} /></span>
			{label}
			{#if elapsed}<span class="font-mono text-label text-text-3">{elapsed}</span>{/if}
		</span>
		{@render body()}
	{:else}
		<button
			type="button"
			aria-expanded={open}
			aria-controls={id}
			onclick={() => (open = !open)}
			class="inline-flex cursor-pointer items-center gap-1.5 self-start rounded-xs py-1 text-ui text-text-2 outline-offset-2 hover:text-text"
		>
			<span class="inline-flex text-text-3"><Icon icon={Brain} size={14} /></span>
			{label}
			<span class="inline-flex text-text-3"><Icon icon={open ? ChevronDown : ChevronRight} size={12} /></span>
		</button>
		{#if open}{@render body()}{/if}
	{/if}
</div>
