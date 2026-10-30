<script lang="ts">
	import { Play, Send, Sparkles } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import RunOptions from '../chat/RunOptions.svelte';

	// Build → Build estimate: shown before anything runs, like a Claude plan. Which project and what
	// it is, the steps Koda will take, what you'll have at the end, then one clear choice; how long
	// and what it costs sits quietly underneath, never the headline.
	type Option = { title: string; detail?: string; recommended?: boolean };
	type Props = {
		project: string;
		summary: string;
		steps: string[];
		outcome: string;
		options: Option[];
		/** e.g. "About 5 min · included in your free plan". */
		cost: string;
		onrun: (choice: { choice: number } | { keepPlanning: string }) => void;
	};
	let { project, summary, steps, outcome, options, cost, onrun }: Props = $props();

	const id = $props.id();
	let value = $state(-1);
	let changes = $state('');

	const chosen = $derived(value >= 0 ? value : Math.max(0, options.findIndex((option) => option.recommended)));
	const keep = $derived(chosen === options.length);
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md bg-surface outline -outline-offset-1 outline-border">
	<header class="flex items-center gap-2.5 border-b border-border px-4 pt-3.5 pb-3.25">
		<Mascot agent="koda" size={20} />
		<span class="text-ui font-medium text-text">Koda</span>
		<span class="min-w-0 flex-1 font-mono text-label text-text-3">{m.build_plan_kind()}</span>
	</header>
	<div class="flex flex-col gap-1 px-4 pt-3.5 pb-1">
		<span class="font-mono text-label text-text-3">{project}</span>
		<h3 id="{id}-title" class="text-body font-semibold text-text">{summary}</h3>
	</div>
	<div class="flex flex-col gap-2 px-4 pt-1.5 pb-3.5">
		<p class="text-ui text-text-2">{m.build_plan_label({ name: 'Koda' })}</p>
		<ol class="flex flex-col gap-1.5">
			{#each steps as step, index (index)}
				<li class="flex gap-2.5 text-ui text-text"><span class="w-3.5 shrink-0 font-mono text-text-3">{index + 1}</span>{step}</li>
			{/each}
		</ol>
	</div>
	<div class="px-4 pb-3.5">
		<p class="flex items-center gap-2.5 rounded-sm bg-accent-bg px-3 py-2.5 text-ui text-text">
			<span class="inline-flex shrink-0 text-accent"><Icon icon={Sparkles} size={14} /></span>{outcome}
		</p>
	</div>
	<div class="border-t border-border p-3.5">
		<RunOptions name="{id}-run" label={summary} {options} who="Koda" bind:value bind:changes />
	</div>
	<footer class="flex flex-wrap items-center gap-2 border-t border-border px-3.5 pt-2.25 pb-2.5">
		<span class="min-w-0 flex-1 basis-40 font-mono text-label text-text-3">{cost}</span>
		<span class="ml-auto">
			<Button
				label={keep ? m.plan_approval_send() : m.build_run()}
				iconEnd={keep ? Send : Play}
				disabled={keep && !changes.trim()}
				onclick={() => onrun(keep ? { keepPlanning: changes.trim() } : { choice: chosen })}
			/>
		</span>
	</footer>
</section>
