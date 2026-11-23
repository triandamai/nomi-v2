<script lang="ts">
	import type { Snippet } from 'svelte';

	// Flow / Mobile — You & Settings, step 8 (Plan & usage) → Plan: the plan first (surface, border,
	// radius 16, 16 in, 12 apart): its name at 32 and what it gives at 14, the meter (6 high, the
	// accent over surface-2), what's used and when it resets in mono, then its action. You's own
	// smaller card is PhonePlanCard.
	type Props = {
		plan: string;
		line: string;
		/** 0…1 of the allowance. */
		used: number;
		usedLabel: string;
		resets?: string;
		action?: Snippet;
	};
	let { plan, line, used, usedLabel, resets, action }: Props = $props();

	const percent = $derived(Math.round(Math.min(1, Math.max(0, used)) * 100));
</script>

<section aria-label={plan} class="flex flex-col gap-3 rounded-2xl bg-surface p-4 outline -outline-offset-1 outline-border">
	<div class="flex flex-col gap-1">
		<p class="text-phone-large font-semibold tracking-phone-large text-text">{plan}</p>
		<p class="text-phone-sm text-text-2">{line}</p>
	</div>
	<div role="meter" aria-label={usedLabel} aria-valuemin={0} aria-valuemax={100} aria-valuenow={percent} class="h-1.5 overflow-hidden rounded-full bg-surface-2">
		<div class="h-full rounded-full bg-accent" style:width="{Math.max(percent, used > 0 ? 2 : 0)}%"></div>
	</div>
	<div class="flex flex-wrap items-center justify-between gap-x-3 gap-y-1 font-mono text-label">
		<span class="text-text-2">{usedLabel}</span>
		{#if resets}<span class="text-text-3">{resets}</span>{/if}
	</div>
	{#if action}<div class="flex flex-col">{@render action()}</div>{/if}
</section>
