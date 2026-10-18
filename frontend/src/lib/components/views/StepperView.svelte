<!-- Steps of a process, joined by a line, each marked with where it stands. -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import ViewFrame from './ViewFrame.svelte';
	import type { StepStatus, StepperView, ViewProps } from '$lib/views/types';

	let { view }: ViewProps<StepperView> = $props();

	const STATUS: Record<StepStatus, () => string> = {
		done: m.view_step_done,
		current: m.view_step_current,
		upcoming: m.view_step_upcoming,
		failed: m.view_step_failed,
		skipped: m.view_step_skipped,
	};
</script>

<ViewFrame title={view.title}>
	<ol class="steps">
		{#each view.steps as step, i (i)}
			{@const status = step.status ?? 'upcoming'}
			<li class="step step--{status}" aria-current={status === 'current' ? 'step' : undefined}>
				<span class="step__dot" aria-hidden="true">
					{#if status === 'done'}<svg width="14" height="14" viewBox="0 0 24 24"><path d="M5 12.5l4.5 4.5L19 7.5" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" /></svg>
					{:else if status === 'failed'}<svg width="14" height="14" viewBox="0 0 24 24"><path d="M7 7l10 10M17 7L7 17" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" /></svg>
					{:else}{i + 1}{/if}
				</span>
				<span class="step__text">
					<span class="step__title">{step.title}<span class="sr-only"> ({STATUS[status]()})</span></span>
					{#if step.detail}<span class="step__detail">{step.detail}</span>{/if}
				</span>
			</li>
		{/each}
	</ol>
</ViewFrame>

<style>
	.steps {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.step {
		position: relative;
		display: flex;
		gap: 12px;
		padding-bottom: 16px;
	}
	.step:last-child {
		padding-bottom: 0;
	}
	/* The line to the next step. */
	.step:not(:last-child)::after {
		content: '';
		position: absolute;
		left: 13px;
		top: 30px;
		bottom: 2px;
		width: 2px;
		border-radius: 1px;
		background: var(--md-sys-color-outline-variant);
	}
	.step--done:not(:last-child)::after {
		background: var(--md-sys-color-primary);
	}
	.step__dot {
		flex: none;
		display: grid;
		place-items: center;
		width: 28px;
		height: 28px;
		border-radius: var(--md-sys-shape-corner-full);
		box-shadow: inset 0 0 0 2px var(--md-sys-color-outline-variant);
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		font-weight: 700;
		font-variant-numeric: tabular-nums;
	}
	.step--done .step__dot {
		box-shadow: none;
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
	.step--current .step__dot {
		box-shadow: inset 0 0 0 2px var(--md-sys-color-primary);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.step--failed .step__dot {
		box-shadow: none;
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}
	.step__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
		padding-top: 3px;
	}
	.step__title {
		font-weight: 600;
		line-height: 1.4;
	}
	.step--upcoming .step__title,
	.step--skipped .step__title {
		color: var(--md-sys-color-on-surface-variant);
		font-weight: 500;
	}
	.step--skipped .step__title {
		text-decoration: line-through;
	}
	.step__detail {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		line-height: 1.4;
	}
</style>
