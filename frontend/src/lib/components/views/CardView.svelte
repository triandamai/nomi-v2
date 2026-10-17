<!-- One thing with its key facts: a title, the figure that matters, a few labelled values, and
     buttons that answer for the person (each sends its message). -->
<script lang="ts">
	import { useChatSend } from '$lib/chatSend';
	import ViewFrame from './ViewFrame.svelte';
	import Value from './Value.svelte';
	import type { CardView, ViewProps } from '$lib/views/types';

	let { view, answered = false }: ViewProps<CardView> = $props();

	const send = useChatSend();
	let sent = $state<string | null>(null);
	const done = $derived(answered || sent !== null);

	function act(message: string) {
		if (done || !send) return;
		send(message);
		sent = message;
	}
</script>

<ViewFrame>
	<div class="card card--{view.tone ?? 'neutral'}">
		<div class="card__head">
			{#if view.eyebrow}<span class="card__eyebrow">{view.eyebrow}</span>{/if}
			<span class="card__title">{view.title}</span>
			{#if view.subtitle}<span class="card__subtitle">{view.subtitle}</span>{/if}
		</div>

		{#if view.highlight}
			<div class="card__highlight">
				<span class="card__highlight-label">{view.highlight.label}</span>
				<span class="card__highlight-value"><Value value={view.highlight.value} format={view.highlight.format} /></span>
			</div>
		{/if}

		{#if view.fields?.length}
			<dl class="card__fields">
				{#each view.fields as field, i (i)}
					<div class="card__field">
						<dt>{field.label}</dt>
						<dd><Value value={field.value} format={field.format} /></dd>
					</div>
				{/each}
			</dl>
		{/if}

		{#if view.body}<p class="card__body">{view.body}</p>{/if}

		{#if view.link || view.actions?.length}
			<div class="card__actions">
				{#if view.link}
					{@const external = !view.link.url.startsWith('/')}
					<a class="card__button card__button--link" href={view.link.url} target={external ? '_blank' : undefined} rel={external ? 'noopener noreferrer' : undefined}>{view.link.label}</a>
				{/if}
				{#each view.actions ?? [] as action, i (i)}
					<button type="button" class="card__button" class:card__button--picked={sent === action.message} disabled={done || !send} onclick={() => act(action.message)}>
						{action.label}
					</button>
				{/each}
			</div>
		{/if}
	</div>
</ViewFrame>

<style>
	.card {
		display: flex;
		flex-direction: column;
		gap: 14px;
	}
	.card__head {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.card__eyebrow {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.card__title {
		font-size: 1.0625rem;
		font-weight: 700;
		line-height: 1.35;
	}
	.card__subtitle {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.card__highlight {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.card__highlight-label {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.card__highlight-value {
		font-family: var(--md-ref-typeface-brand);
		font-size: 2rem;
		font-weight: 800;
		letter-spacing: -0.02em;
		line-height: 1.1;
	}
	.card--positive .card__highlight-value {
		color: var(--md-sys-color-primary);
	}
	.card--negative .card__highlight-value {
		color: var(--md-sys-color-error);
	}
	.card--warning .card__highlight-value {
		color: var(--md-sys-color-tertiary);
	}
	.card__fields {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 4px 12px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-low);
	}
	.card__field {
		display: flex;
		justify-content: space-between;
		gap: 16px;
		padding: 8px 0;
		border-top: 1px solid var(--md-sys-color-outline-variant);
		font-size: 0.9375rem;
	}
	.card__field:first-child {
		border-top: none;
	}
	.card__field dt {
		color: var(--md-sys-color-on-surface-variant);
	}
	.card__field dd {
		margin: 0;
		text-align: end;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
		overflow-wrap: anywhere;
	}
	.card__body {
		margin: 0;
		line-height: 1.5;
		white-space: pre-line;
	}
	.card__actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.card__button {
		display: inline-flex;
		align-items: center;
		min-height: 40px;
		padding: 0 16px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font: inherit;
		font-weight: 600;
		font-size: 0.9375rem;
		text-decoration: none;
		cursor: pointer;
		transition: background-color var(--nomi-motion-effects-fast);
	}
	.card__button--link {
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
	.card__button:hover:not(:disabled) {
		background: color-mix(in srgb, var(--md-sys-color-on-secondary-container) 10%, var(--md-sys-color-secondary-container));
	}
	.card__button--link:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-primary) 10%, var(--md-sys-color-primary)) !important;
	}
	.card__button:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.card__button:disabled {
		cursor: default;
		opacity: 0.5;
	}
	.card__button--picked:disabled {
		opacity: 1;
	}
	@media (max-width: 600px) {
		.card__button {
			min-height: 44px;
		}
	}
</style>
