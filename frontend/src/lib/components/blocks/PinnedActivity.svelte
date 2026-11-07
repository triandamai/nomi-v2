<script lang="ts">
	// The crew's work on this turn, pinned just above the composer (see $lib/pinnedActivity): who's
	// on it and since when, what it's doing right now with the time so far, how far through its
	// to-do it is, and the one thing to do about it: stop it while it works, go to what it's
	// waiting on, or see why it failed. The steps it took fold underneath.
	import { Check, CircleAlert, LoaderCircle, MessageCircleQuestion, Square, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';
	import Button from '$lib/components/ds/actions/Button.svelte';
	import IconButton from '$lib/components/ds/actions/IconButton.svelte';
	import Mascot from '$lib/components/ds/agents/Mascot.svelte';
	import ThoughtSteps from '$lib/components/ds/chat/ThoughtSteps.svelte';
	import type { CrewId } from '$lib/components/ds/agents/crew';
	import type { StepState } from '$lib/components/ds/chat/stepState';
	import type { TodoProgress } from '$lib/pinnedActivity';

	type Props = {
		status: 'working' | 'done' | 'failed' | 'waiting';
		/** Who's on it, and their mascot. */
		who?: string;
		agent?: CrewId;
		/** The quiet line by their name: "running · started 08:13". */
		meta?: string;
		/** What it's doing right now, or how it ended. */
		headline: string;
		elapsed?: string | null;
		/** How far through its to-do it is, when it keeps one. */
		progress?: TodoProgress | null;
		/** About how long is left ("~3 min left"). */
		eta?: string | null;
		steps: { label: string; state: StepState }[];
		/** Stops the crew (working only). */
		onstop?: () => void;
		/** Brings what it's waiting on, or the failure, into view. */
		onjump?: () => void;
		jumpLabel?: string;
		ondismiss?: () => void;
	};
	let { status, who, agent, meta, headline, elapsed, progress = null, eta = null, steps, onstop, onjump, jumpLabel, ondismiss }: Props = $props();

	let open = $state(false);
	const ICON = { working: LoaderCircle, done: Check, failed: CircleAlert, waiting: MessageCircleQuestion } as const;

	/** Each to-do item as a segment: done, the one on now, or still to come. */
	const segments = $derived(
		progress
			? Array.from({ length: progress.total }, (_, index) =>
					index < progress.done ? 'done' : index === progress.position - 1 && progress.done < progress.total ? 'now' : 'todo',
				)
			: [],
	);
</script>

<section class="pinned" data-status={status} aria-live="polite">
	{#if who}
		<header class="pinned__head">
			{#if agent}<Mascot {agent} size={20} />{/if}
			<span class="pinned__who">{who}</span>
			{#if meta}<span class="pinned__meta">{meta}</span>{/if}
			{#if ondismiss}<span class="pinned__close"><IconButton icon={X} label={m.pinned_dismiss()} variant="ghost" iconSize={14} onclick={ondismiss} /></span>{/if}
		</header>
	{/if}
	<div class="pinned__now">
		<span class={['pinned__icon', status === 'working' && 'animate-spin motion-reduce:animate-none']}><Icon icon={ICON[status]} size={16} /></span>
		<span role="status" class="pinned__headline">{headline}</span>
		{#if elapsed}<span class="pinned__elapsed">{elapsed}</span>{/if}
		{#if status === 'working' && onstop}
			<Button label={m.pinned_stop()} variant="ghost" iconStart={Square} onclick={onstop} aria-keyshortcuts="Escape" />
		{:else if onjump && jumpLabel}
			<Button label={jumpLabel} variant="ghost" onclick={onjump} />
		{/if}
		{#if ondismiss && !who}<IconButton icon={X} label={m.pinned_dismiss()} variant="ghost" iconSize={14} onclick={ondismiss} />{/if}
	</div>
	{#if progress}
		<div
			class="pinned__bar"
			role="progressbar"
			aria-label={m.pinned_progress({ position: progress.position, total: progress.total })}
			aria-valuemin={0}
			aria-valuemax={progress.total}
			aria-valuenow={progress.done}
		>
			{#each segments as segment, index (index)}<span class="pinned__segment" data-state={segment}></span>{/each}
		</div>
		<div class="pinned__foot">
			<span class="pinned__step">
				{m.pinned_progress({ position: progress.position, total: progress.total })}{#if progress.current}<span class="pinned__current"> · {progress.current}</span>{/if}
			</span>
			{#if eta}<span class="pinned__eta">{eta}</span>{/if}
		</div>
	{/if}
	{#if steps.length > 0}
		<div class="pinned__steps">
			<ThoughtSteps bind:open label={steps.length === 1 ? m.activity_step_one() : m.activity_steps({ count: steps.length })} {steps} />
		</div>
	{/if}
</section>

<style>
	.pinned {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 10px 12px 12px;
		border-radius: 16px;
		border: 1px solid var(--md-sys-color-outline-variant);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.pinned__head {
		display: flex;
		align-items: center;
		gap: 8px;
		min-height: 28px;
		min-width: 0;
	}
	.pinned__who {
		flex-shrink: 0;
		font-size: 0.9375rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}
	.pinned__meta {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.pinned__close {
		margin-left: auto;
		flex-shrink: 0;
	}
	.pinned__now {
		display: flex;
		align-items: center;
		gap: 10px;
		min-height: 44px;
		padding: 4px 4px 4px 12px;
		border-radius: 12px;
		border: 1px solid var(--md-sys-color-outline-variant);
		background: var(--md-sys-color-surface-container-low);
	}
	.pinned__icon {
		display: inline-flex;
		flex-shrink: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.pinned[data-status='done'] .pinned__icon {
		color: var(--md-sys-color-primary);
	}
	.pinned[data-status='failed'] .pinned__icon {
		color: var(--md-sys-color-error);
	}
	.pinned[data-status='waiting'] .pinned__icon {
		color: var(--md-sys-color-tertiary);
	}
	.pinned__headline {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface);
	}
	.pinned[data-status='working'] .pinned__headline {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
	}
	.pinned__elapsed {
		flex-shrink: 0;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		font-variant-numeric: tabular-nums;
		color: var(--md-sys-color-on-surface-variant);
	}
	.pinned__bar {
		display: flex;
		gap: 4px;
		padding: 0 2px;
	}
	.pinned__segment {
		flex: 1;
		height: 4px;
		border-radius: 999px;
		background: var(--md-sys-color-outline-variant);
	}
	.pinned__segment[data-state='done'] {
		background: var(--md-sys-color-primary);
	}
	.pinned__segment[data-state='now'] {
		background: color-mix(in srgb, var(--md-sys-color-primary) 45%, var(--md-sys-color-outline-variant));
	}
	.pinned__foot {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
		padding: 0 2px;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.pinned__step {
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.pinned__eta {
		flex-shrink: 0;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
	}
	.pinned__steps {
		max-height: 40vh;
		overflow-y: auto;
		padding-right: 4px;
	}
</style>
