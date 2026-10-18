<!-- A choice between a few options, each with what speaks for and against it. Picking one sends
     it as the person's answer. -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { useChatSend } from '$lib/chatSend';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import ViewFrame from './ViewFrame.svelte';
	import type { DecisionView, ViewProps } from '$lib/views/types';

	let { view, agent = null, answered = false }: ViewProps<DecisionView> = $props();

	const send = useChatSend();
	let picked = $state<string | null>(null);
	const done = $derived(answered || picked !== null);

	function choose(label: string) {
		if (done || !send) return;
		send(label);
		picked = label;
	}
</script>

<ViewFrame>
	<header class="decision__head">
		<AgentShape {agent} size={28} face />
		<span class="decision__question">{view.question}</span>
	</header>
	<div class="decision__options">
		{#each view.options as option, i (i)}
			<button
				type="button"
				class="option"
				class:option--picked={picked === option.label}
				class:option--recommended={option.recommended}
				disabled={done || !send}
				onclick={() => choose(option.label)}
			>
				<span class="option__top">
					<span class="option__label">{option.label}</span>
					{#if option.recommended}<span class="option__badge">{m.view_recommended()}</span>{/if}
				</span>
				{#if option.detail}<span class="option__detail">{option.detail}</span>{/if}
				{#if option.pros?.length || option.cons?.length}
					<ul class="option__points">
						{#each option.pros ?? [] as pro, p (p)}
							<li class="point point--pro"><span class="sr-only">{m.view_pro()}: </span>{pro}</li>
						{/each}
						{#each option.cons ?? [] as con, c (c)}
							<li class="point point--con"><span class="sr-only">{m.view_con()}: </span>{con}</li>
						{/each}
					</ul>
				{/if}
			</button>
		{/each}
	</div>
	{#if done}<p class="decision__done">{m.view_answered()}</p>{/if}
</ViewFrame>

<style>
	.decision__head {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.decision__question {
		font-weight: 700;
		line-height: 1.35;
	}
	.decision__options {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(100%, 200px), 1fr));
		gap: 8px;
	}
	.option {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 6px;
		padding: 12px 14px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-low);
		color: inherit;
		font: inherit;
		text-align: start;
		cursor: pointer;
		transition: background-color var(--nomi-motion-effects-fast), border-radius var(--nomi-motion-spatial-fast);
	}
	.option:hover:not(:disabled) {
		background: var(--md-sys-color-surface-container-high);
		border-radius: var(--md-sys-shape-corner-extra-large);
	}
	.option:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.option--recommended {
		border-color: var(--md-sys-color-primary);
	}
	.option:disabled {
		cursor: default;
		opacity: 0.55;
	}
	.option--picked:disabled {
		opacity: 1;
		border-color: transparent;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.option__top {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
	}
	.option__label {
		font-weight: 700;
	}
	.option__badge {
		padding: 2px 8px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
		font-size: 0.75rem;
		font-weight: 600;
	}
	.option__detail {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		line-height: 1.4;
	}
	.option--picked .option__detail {
		color: inherit;
	}
	.option__points {
		display: flex;
		flex-direction: column;
		gap: 3px;
		margin: 2px 0 0;
		padding: 0;
		list-style: none;
		font-size: 0.875rem;
		line-height: 1.35;
	}
	.point {
		display: flex;
		gap: 6px;
	}
	.point::before {
		flex: none;
		width: 1em;
		font-weight: 700;
	}
	.point--pro::before {
		content: '+';
		color: var(--md-sys-color-primary);
	}
	.point--con::before {
		content: '−';
		color: var(--md-sys-color-error);
	}
	.decision__done {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
</style>
