<!-- One result that should stand out: a large value, what it is, and a line of context. -->
<script lang="ts">
	import ViewFrame from './ViewFrame.svelte';
	import Value from './Value.svelte';
	import type { AnswerView, ViewProps } from '$lib/views/types';

	let { view }: ViewProps<AnswerView> = $props();
</script>

<ViewFrame>
	<div class="answer answer--{view.tone ?? 'neutral'}">
		{#if view.label}<span class="answer__label">{view.label}</span>{/if}
		<span class="answer__value"><Value value={view.value} format={view.format} /></span>
		{#if view.detail}<span class="answer__detail">{view.detail}</span>{/if}
	</div>
</ViewFrame>

<style>
	.answer {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}
	.answer__label {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.answer__value {
		font-family: var(--md-ref-typeface-brand);
		font-size: clamp(1.75rem, 6vw, 2.5rem);
		font-weight: 800;
		letter-spacing: -0.025em;
		line-height: 1.1;
		overflow-wrap: anywhere;
	}
	.answer--positive .answer__value {
		color: var(--md-sys-color-primary);
	}
	.answer--negative .answer__value {
		color: var(--md-sys-color-error);
	}
	.answer--warning .answer__value {
		color: var(--md-sys-color-tertiary);
	}
	.answer__detail {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.9375rem;
		line-height: 1.45;
	}
</style>
