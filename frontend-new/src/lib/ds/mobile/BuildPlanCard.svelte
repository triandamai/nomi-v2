<script lang="ts">
	import { ArrowRight, Sparkles } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import PhoneButton from './PhoneButton.svelte';

	// Mobile / Build Plan: Koda's plan on a phone. The steps, what you get, one big Build it with
	// what it costs under it, and the two other choices side by side.
	type Props = {
		project: string;
		summary: string;
		steps: string[];
		outcome: string;
		cost: string;
		onbuild: () => void;
		onaskfirst: () => void;
		onkeep: () => void;
	};
	let { project, summary, steps, outcome, cost, onbuild, onaskfirst, onkeep }: Props = $props();
</script>

<article class="flex w-full flex-col gap-3.5 rounded-3xl border border-border bg-surface p-3.75">
	<p class="flex items-center gap-2">
		<Mascot agent="koda" size={20} />
		<span class="text-phone-sm font-medium text-text">Koda</span>
		<span class="font-mono text-label text-text-3">{m.build_plan_kind()}</span>
	</p>
	<div class="flex flex-col gap-1">
		<span class="font-mono text-label text-text-3">{project}</span>
		<h3 class="text-phone-heading leading-firm font-semibold text-text">{summary}</h3>
	</div>
	<ol class="flex flex-col gap-2">
		{#each steps as step, index (index)}
			<li class="flex gap-2.5">
				<span class="w-3.5 shrink-0 font-mono text-ui text-text-3">{index + 1}</span>
				<span class="min-w-0 flex-1 text-body leading-snug text-text">{step}</span>
			</li>
		{/each}
	</ol>
	<p class="flex items-center gap-2.5 rounded-lg bg-accent-bg px-3 py-2.5">
		<span class="inline-flex shrink-0 text-accent"><Icon icon={Sparkles} size={14} /></span>
		<span class="min-w-0 flex-1 text-phone-sm leading-snug text-text">{outcome}</span>
	</p>
	<PhoneButton label={m.build_run()} icon={ArrowRight} onclick={onbuild} />
	<p class="text-center text-caption text-text-3">{cost}</p>
	<div class="flex gap-2">
		<button type="button" class="flex h-11 flex-1 cursor-pointer items-center justify-center rounded-md border border-border-strong text-phone-sm text-text outline-offset-2" onclick={onaskfirst}
			>{m.bplan_ask_first()}</button
		>
		<button type="button" class="flex h-11 flex-1 cursor-pointer items-center justify-center rounded-md border border-border-strong text-phone-sm text-text outline-offset-2" onclick={onkeep}
			>{m.plan_approval_keep()}</button
		>
	</div>
</article>
