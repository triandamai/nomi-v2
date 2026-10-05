<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		headline,
		supportingText,
		leading,
		trailing,
		selected = false,
		class: extraClass = '',
	}: {
		headline: string;
		supportingText?: string;
		/** An avatar, shape or checkbox before the text. */
		leading?: Snippet;
		trailing?: Snippet;
		selected?: boolean;
		class?: string;
	} = $props();
</script>

<div role="listitem" class="m3-list-item {extraClass}" class:m3-list-item--selected={selected}>
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
</div>

<style>
	.m3-list-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
		padding: 10px 14px;
		border-radius: var(--md-sys-shape-corner-large-increased);
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
