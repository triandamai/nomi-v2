<!-- Next things the person might ask, as chips: tapping one sends it as their message. -->
<script lang="ts">
	import { useChatSend } from '$lib/chatSend';
	import type { SuggestionsView, ViewProps } from '$lib/views/types';

	let { view, answered = false }: ViewProps<SuggestionsView> = $props();

	const send = useChatSend();
	let sent = $state<string | null>(null);
	const done = $derived(answered || sent !== null);

	function pick(item: string) {
		if (done || !send) return;
		send(item);
		sent = item;
	}
</script>

<!-- Once the conversation has moved on, the chips have done their job: they go. -->
{#if !answered || sent}
	<div class="suggestions" role="group" aria-label={view.title}>
		{#if view.title}<span class="suggestions__title">{view.title}</span>{/if}
		<div class="suggestions__chips">
			{#each view.items as item, i (i)}
				<button type="button" class="chip" class:chip--picked={sent === item} disabled={done || !send} onclick={() => pick(item)}>
					<svg width="16" height="16" viewBox="0 0 24 24" aria-hidden="true"><path d="M5 12h12M13 7l5 5-5 5" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" /></svg>
					{item}
				</button>
			{/each}
		</div>
	</div>
{/if}

<style>
	.suggestions {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	.suggestions__title {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.suggestions__chips {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		max-width: 100%;
		min-height: 36px;
		padding: 6px 14px 6px 10px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-small);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: 0.9375rem;
		line-height: 1.3;
		text-align: start;
		cursor: pointer;
		transition: background-color var(--nomi-motion-effects-fast), border-radius var(--nomi-motion-spatial-fast);
	}
	.chip svg {
		flex: none;
		color: var(--md-sys-color-primary);
	}
	.chip:hover:not(:disabled) {
		background: var(--md-sys-color-surface-container-high);
		border-radius: var(--md-sys-shape-corner-medium);
	}
	.chip:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.chip:disabled {
		cursor: default;
		opacity: 0.5;
	}
	.chip--picked:disabled {
		opacity: 1;
		border-color: transparent;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	@media (max-width: 600px) {
		.chip {
			min-height: 44px;
		}
	}
</style>
