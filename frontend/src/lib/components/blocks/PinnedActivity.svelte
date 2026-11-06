<script lang="ts">
	// The crew's work on this turn, pinned just above the composer (see $lib/pinnedActivity): what
	// it's doing now with the time so far, the steps it took (folded), and the one thing to do
	// about it: stop it while it works, go to what it's waiting on, or see why it failed.
	import { Check, CircleAlert, LoaderCircle, MessageCircleQuestion, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '$lib/components/ds/foundations/Icon.svelte';
	import Button from '$lib/components/ds/actions/Button.svelte';
	import IconButton from '$lib/components/ds/actions/IconButton.svelte';
	import StopHint from '$lib/components/ds/chat/StopHint.svelte';
	import ThoughtSteps from '$lib/components/ds/chat/ThoughtSteps.svelte';
	import type { StepState } from '$lib/components/ds/chat/stepState';

	type Props = {
		status: 'working' | 'done' | 'failed' | 'waiting';
		headline: string;
		elapsed?: string | null;
		steps: { label: string; state: StepState }[];
		/** Stops the crew (working only). */
		onstop?: () => void;
		/** Brings what it's waiting on, or the failure, into view. */
		onjump?: () => void;
		jumpLabel?: string;
		ondismiss?: () => void;
	};
	let { status, headline, elapsed, steps, onstop, onjump, jumpLabel, ondismiss }: Props = $props();

	let open = $state(false);
	const ICON = { working: LoaderCircle, done: Check, failed: CircleAlert, waiting: MessageCircleQuestion } as const;
</script>

<section class="pinned" data-status={status} aria-live="polite">
	<div class="pinned__row">
		<span class={['pinned__icon', status === 'working' && 'animate-spin motion-reduce:animate-none']}><Icon icon={ICON[status]} size={16} /></span>
		<span role="status" class="pinned__headline">{headline}</span>
		{#if elapsed}<span class="pinned__elapsed">{elapsed}</span>{/if}
		{#if status === 'working' && onstop}
			<StopHint {onstop} />
		{:else if onjump && jumpLabel}
			<Button label={jumpLabel} variant="ghost" onclick={onjump} />
		{/if}
		{#if ondismiss}<IconButton icon={X} label={m.pinned_dismiss()} variant="ghost" iconSize={14} onclick={ondismiss} />{/if}
	</div>
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
		gap: 4px;
		padding: 8px 8px 8px 12px;
		border-radius: 16px;
		border: 1px solid var(--md-sys-color-outline-variant);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.pinned__row {
		display: flex;
		align-items: center;
		gap: 8px;
		min-height: 36px;
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
	.pinned__elapsed {
		flex-shrink: 0;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.pinned__steps {
		max-height: 40vh;
		overflow-y: auto;
		padding-right: 4px;
	}
</style>
