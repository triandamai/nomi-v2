<script lang="ts">
	import { FilePenLine } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Tag from '../status/Tag.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';
	import { stepNumber } from '../chat/stepState';

	// Flow / Mobile Chat — Clarify → plan (steps 6, 9) and Plan → approve → act (step 1): a plan the
	// crew proposed, on a phone, drawn as the pen's Chat / Draft Plan with what a phone can't do
	// switched off. The header ("Draft plan · {title}") with DRAFT, or V{n} once revised; the intro;
	// the numbered steps, each with its owner (no reorder, reassign or remove: the person tells Nomi
	// what to change, UX contract G3); a step a revision added marked "+" on accent-bg; and the
	// estimate in the foot only when there is one.
	type Step = { label: string; owner: { agent: CrewId; name: string }; added?: boolean };
	type Props = { title: string; version?: number; intro: string; steps: Step[]; estimate?: string };
	let { title, version, intro, steps, estimate }: Props = $props();
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md bg-bg outline -outline-offset-1 outline-border-strong">
	<header class="flex items-center gap-2.5 border-b border-border bg-surface px-3.5 pt-2.5 pb-2.25">
		<span class="inline-flex text-text-2"><Icon icon={FilePenLine} size={14} /></span>
		<h3 class="min-w-0 flex-1 truncate text-ui font-medium text-text">{m.phone_plan_draft_title({ title })}</h3>
		<Tag kind="neutral" label={version && version > 1 ? m.phone_plan_version({ version: String(version) }) : m.draft_tag()} />
	</header>
	<p class="px-3.5 pt-3 pb-1 text-ui leading-normal text-text-2">{intro}</p>
	<ol class="flex flex-col px-3.5 py-1">
		{#each steps as step, index (index)}
			<li class={['flex items-center gap-2.5 py-2', step.added && 'rounded-sm bg-accent-bg']} aria-label={m.phone_plan_step({ n: String(index + 1), owner: step.owner.name })}>
				{#if step.added}
					<span class="font-mono text-label text-accent">+<span class="sr-only">{m.phone_plan_added()}</span></span>
				{:else}
					<span class="font-mono text-label text-text-3">{stepNumber(index)}</span>
				{/if}
				<span class="min-w-0 flex-1 text-ui text-text">{step.label}</span>
				<span class="inline-flex h-6.5 shrink-0 items-center gap-1.5 rounded-sm pr-2 pl-1 outline -outline-offset-1 outline-border-strong">
					<Mascot agent={step.owner.agent} size={20} />
					<span class="text-caption text-text-2">{step.owner.name}</span>
				</span>
			</li>
		{/each}
	</ol>
	{#if estimate}
		<footer class="border-t border-border px-3.5 pt-2.25 pb-2.5 font-mono text-label text-text-3">{estimate}</footer>
	{/if}
</section>
