<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		headline,
		supportingText,
		leading,
		trailing,
		selected = false,
		onclick,
		label,
		class: extraClass = '',
	}: {
		headline: string;
		supportingText?: string;
		/** An avatar, shape or checkbox before the text. */
		leading?: Snippet;
		trailing?: Snippet;
		selected?: boolean;
		/** Makes the whole row one button (opening its details, say). */
		onclick?: () => void;
		/** What the button says to screen readers, when the headline alone isn't enough. */
		label?: string;
		class?: string;
	} = $props();
</script>

{#snippet content()}
	{#if leading}
		<div class="m3-list-item__leading">{@render leading()}</div>
	{/if}
	<div class="m3-list-item__text">
		<p class="md-body-large" style="margin: 0; color: var(--md-sys-color-on-surface)">{headline}</p>
		{#if supportingText}
			<p class="md-body-small" style="margin: 0; color: var(--md-sys-color-on-surface-variant)">{supportingText}</p>
		{/if}
	</div>
	{#if trailing}
		<div class="m3-list-item__trailing">{@render trailing()}</div>
	{/if}
{/snippet}

{#if onclick}
	<div role="listitem" class="m3-list-item-wrap {extraClass}">
		<button type="button" class="m3-list-item m3-list-item--action" class:m3-list-item--selected={selected} aria-label={label} {onclick}>
			{@render content()}
		</button>
	</div>
{:else}
	<div role="listitem" class="m3-list-item {extraClass}" class:m3-list-item--selected={selected}>
		{@render content()}
	</div>
{/if}

<style>
	.m3-list-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 10px 14px;
		border-radius: var(--md-sys-shape-corner-large-increased);
	}
	.m3-list-item--action {
		width: 100%;
		border: none;
		background: transparent;
		color: inherit;
		font: inherit;
		text-align: left;
		cursor: pointer;
		transition:
			background-color var(--nomi-motion-effects-fast, 150ms),
			border-radius var(--nomi-motion-spatial-fast, 200ms);
	}
	.m3-list-item--action:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.m3-list-item--action:active {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 12%, transparent);
		border-radius: var(--md-sys-shape-corner-extra-large);
	}
	.m3-list-item--action:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -2px;
	}
	.m3-list-item--selected {
		background: var(--md-sys-color-primary-container);
	}
	.m3-list-item__text {
		min-width: 0;
		flex: 1;
	}
	.m3-list-item__text p {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.m3-list-item__trailing {
		flex-shrink: 0;
	}
	.m3-list-item__leading {
		flex-shrink: 0;
		display: flex;
		align-items: center;
		margin-right: 6px;
	}
</style>
