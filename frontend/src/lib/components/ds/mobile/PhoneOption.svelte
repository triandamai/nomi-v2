<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Tag from '../status/Tag.svelte';

	// Mobile / Option: one answer on a phone, a 14pt-round card with its number, title at 16pt, the
	// tradeoff, and a 22pt radio on the right. The chosen one takes the accent.
	type Props = { name: string; key?: number; title: string; detail?: string; recommended?: boolean; checked: boolean; onselect: () => void };
	let { name, key, title, detail, recommended = false, checked, onselect }: Props = $props();
</script>

<label class={['flex w-full cursor-pointer items-center gap-3 rounded-xl border p-3.25', checked ? 'border-accent bg-accent-bg' : 'border-border-strong']}>
	{#if key !== undefined}
		<span aria-hidden="true" class="inline-flex size-6.5 shrink-0 items-center justify-center rounded-full bg-surface-2 font-mono text-caption text-text-2">{key}</span>
	{/if}
	<span class="flex min-w-0 flex-1 flex-col gap-0.75">
		<span class="text-phone font-medium text-text">{title}</span>
		{#if detail}<span class="text-ui leading-snug text-text-3">{detail}</span>{/if}
		{#if recommended}<span><Tag kind="go" label={m.choice_recommended()} /></span>{/if}
	</span>
	<input
		type="radio"
		{name}
		{checked}
		onchange={onselect}
		class="radio size-5.5 shrink-0 cursor-pointer appearance-none rounded-full border-border-strong outline-offset-2 checked:border-accent"
	/>
</label>

<style>
	/* The pen's radio ring is 1.5pt; chosen, it closes to a 6pt mint ring. */
	.radio {
		border-width: 1.5px;
	}
	.radio:checked {
		border-width: 6px;
	}
</style>
