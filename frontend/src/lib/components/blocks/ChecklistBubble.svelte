<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import type { Snippet } from 'svelte';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import WavyProgress from '$lib/components/m3/WavyProgress.svelte';
	import { agentLook, GRADIENT_STOPS, TONE_ACCENT } from '$lib/components/m3/shapes';
	import type { ChecklistItem } from '$lib/planChecklist';

	// The chat's draft / to-do bubble (design: "Planning's draft 3 / 5"): the posting agent's
	// shape and name, a wavy progress line and a checklist, tinted in that agent's gradient.

	let {
		agent,
		heading,
		items = [],
		maxItems,
		children,
		footer,
		ontoggle,
	}: {
		/** Who posted it (display name or type); picks the shape and the tint. */
		agent: string | null;
		heading: string;
		items?: ChecklistItem[];
		/** Show at most this many items, then "N more". */
		maxItems?: number;
		/** Anything between the progress line and the list (a plan's title or excerpt). */
		children?: Snippet;
		footer?: Snippet;
		/** Makes each item a checkbox the person can tick; gets the item's index and new state. */
		ontoggle?: (index: number, done: boolean) => void;
	} = $props();


	const look = $derived(agentLook(agent));
	const tint = $derived(GRADIENT_STOPS[look.tone].at(-1));
	const done = $derived(items.filter((item) => item.status === 'done').length);
	const working = $derived(items.some((item) => item.status === 'in_progress'));
	const shown = $derived(maxItems !== undefined ? items.slice(0, maxItems) : items);
	const hidden = $derived(items.length - shown.length);
</script>

{#snippet row(item: ChecklistItem)}
	<span class="checklist__mark" aria-hidden="true">
		{#if item.status === 'done'}
			<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><path d="m5 12 5 5 9-10" /></svg>
		{/if}
	</span>
	<span>{item.text}</span>
	{#if item.status === 'in_progress'}<span class="sr-only">{m.checklist_current()}</span>{/if}
	{#if item.status === 'done' && !ontoggle}<span class="sr-only">{m.checklist_done()}</span>{/if}
{/snippet}

<section class="checklist" aria-label={heading} style="--tint: {tint}; --accent: {TONE_ACCENT[look.tone]}">
	<div class="checklist__head">
		<AgentShape {agent} size={26} {working} />
		<span class="checklist__heading">{heading}</span>
		{#if items.length > 0}
			<span class="nomi-meta checklist__count">{done} / {items.length}</span>
		{/if}
	</div>

	{#if items.length > 0}
		<WavyProgress value={done / items.length} tone={look.tone} label="{done} of {items.length} done" />
	{/if}

	{@render children?.()}

	{#if shown.length > 0}
		<ul class="checklist__items">
			{#each shown as item, i (i)}
				<li class="checklist__item checklist__item--{item.status}">
					{#if ontoggle}
						<button
							type="button"
							role="checkbox"
							class="checklist__check"
							aria-checked={item.status === 'done'}
							onclick={() => ontoggle(i, item.status !== 'done')}
						>
							{@render row(item)}
						</button>
					{:else}
						{@render row(item)}
					{/if}
				</li>
			{/each}
		</ul>
		{#if hidden > 0}
			<span class="nomi-meta checklist__more">+ {hidden} more</span>
		{/if}
	{/if}

	{@render footer?.()}
</section>

<style>
	.checklist {
		display: flex;
		flex-direction: column;
		gap: 12px;
		width: 100%;
		max-width: 540px;
		box-sizing: border-box;
		padding: 20px 22px 16px;
		border-radius: var(--nomi-shape-bubble-start);
		background: color-mix(in srgb, var(--tint) 16%, var(--md-sys-color-surface-container-lowest));
		color: var(--md-sys-color-on-surface);
	}
	.checklist__head {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.checklist__heading {
		flex: 1;
		min-width: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.25rem;
		font-weight: 700;
		letter-spacing: -0.01em;
	}
	.checklist__count {
		color: inherit;
	}
	/* Doubled class: outranks the bubble's markdown list rule (.message-bubble ul). */
	.checklist .checklist__items {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.checklist__item {
		display: flex;
		align-items: center;
		gap: 12px;
		min-height: 38px;
		font-size: 0.9375rem;
		line-height: 1.35;
	}
	.checklist__item--in_progress {
		font-weight: 650;
	}
	.checklist__item--pending {
		color: var(--md-sys-color-on-surface-variant);
	}
	.checklist__mark {
		flex: none;
		display: flex;
		align-items: center;
		justify-content: center;
		width: 22px;
		height: 22px;
		box-sizing: border-box;
		border-radius: 7px;
		border: 2px solid color-mix(in srgb, var(--accent) 45%, transparent);
	}
	.checklist__item--in_progress .checklist__mark {
		border-color: var(--accent);
	}
	.checklist__item--done .checklist__mark {
		border-color: var(--accent);
		background: var(--accent);
		color: #ffffff;
	}
	/* Tickable items: the whole row is the hit target. */
	.checklist__check {
		display: flex;
		align-items: center;
		gap: 12px;
		width: calc(100% + 16px);
		min-height: 38px;
		margin: 0 -8px;
		padding: 0 8px;
		border: none;
		border-radius: 10px;
		background: none;
		color: inherit;
		font: inherit;
		text-align: start;
		cursor: pointer;
	}
	.checklist__check:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 6%, transparent);
	}
	.checklist__check:focus-visible {
		outline: 2px solid var(--accent);
		outline-offset: 1px;
	}
	.checklist__item--done .checklist__check > span:not(.checklist__mark) {
		text-decoration: line-through;
		text-decoration-color: color-mix(in srgb, currentColor 45%, transparent);
	}
	.checklist__more {
		text-transform: none;
	}
</style>
