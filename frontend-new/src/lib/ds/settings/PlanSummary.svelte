<script lang="ts">
	import type { Snippet } from 'svelte';
	import Tag from '../status/Tag.svelte';

	// App / Settings / Plan & usage (C0og49) → Your Plan: the plan the person is on, outlined, 24 in:
	// its name big with the price and allowance beside it, how much of the allowance is used (the
	// accent) and where this pace ends the month (heat-2) on a 6px track, the key under it with
	// when it starts again, then the actions; the billing facts in a 300 column on the surface.
	type Props = {
		label: string;
		tag: string;
		plan: string;
		line: string;
		allowance: { label: string; value: string };
		used: number;
		projected?: number;
		legend: { used: string; projected?: string; resets?: string };
		actions?: Snippet;
		aside?: Snippet;
	};
	let { label, tag, plan, line, allowance, used, projected, legend, actions, aside }: Props = $props();

	const percent = (share: number) => `${Math.round(Math.min(1, Math.max(0, share)) * 1000) / 10}%`;
</script>

<section aria-label={label} class="flex flex-col overflow-hidden rounded-md outline -outline-offset-1 outline-border md:flex-row">
	<div class="flex min-w-0 flex-1 flex-col gap-3 p-6">
		<p class="flex items-center justify-between gap-2">
			<span class="font-mono text-label tracking-label text-text-3 uppercase">{label}</span>
			<Tag kind="neutral" label={tag} />
		</p>
		<p class="flex flex-wrap items-baseline gap-2.5">
			<span class="text-hero leading-none font-medium text-text">{plan}</span>
			<span class="text-body text-text-2">{line}</span>
		</p>
		<p class="flex items-center justify-between gap-2">
			<span class="text-caption text-text-2">{allowance.label}</span>
			<span class="font-mono text-label text-text">{allowance.value}</span>
		</p>
		<span class="relative h-1.5 overflow-hidden rounded-full bg-surface-2" role="meter" aria-label={allowance.label} aria-valuenow={Math.round(used * 100)} aria-valuemin={0} aria-valuemax={100} aria-valuetext={allowance.value}>
			{#if projected !== undefined}<span class="absolute inset-y-0 left-0 rounded-full bg-heat-2" style="width: {percent(projected)}"></span>{/if}
			<span class="absolute inset-y-0 left-0 rounded-full bg-accent" style="width: {percent(used)}"></span>
		</span>
		<p class="flex flex-wrap items-center gap-x-4 gap-y-1 font-mono text-label">
			<span class="flex items-center gap-1.5 text-text-2"><span aria-hidden="true" class="size-2 rounded-cell bg-accent"></span>{legend.used}</span>
			{#if legend.projected}<span class="flex items-center gap-1.5 text-text-2"><span aria-hidden="true" class="size-2 rounded-cell bg-heat-2"></span>{legend.projected}</span>{/if}
			{#if legend.resets}<span class="ml-auto text-text-3">{legend.resets}</span>{/if}
		</p>
		{#if actions}<div class="flex flex-wrap items-center gap-2 pt-1.5">{@render actions()}</div>{/if}
	</div>
	{#if aside}<div class="flex shrink-0 flex-col border-t border-border bg-surface px-5 py-4 md:w-75 md:border-t-0 md:border-l md:pl-4.75">{@render aside()}</div>{/if}
</section>
