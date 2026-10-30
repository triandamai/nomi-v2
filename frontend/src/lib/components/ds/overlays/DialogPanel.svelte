<script lang="ts">
	import type { Snippet } from 'svelte';

	// The M3 dialog box: headline and supporting text over the content, actions at the end.
	// Presentational: Dialog puts it in a native modal; the gallery shows it open.
	type Props = { id: string; title: string; description?: string; wide?: boolean; children?: Snippet; actions?: Snippet };
	let { id, title, description, wide = false, children, actions }: Props = $props();
</script>

<div class="panel" class:panel--wide={wide}>
	<div class="body">
		<h2 id="{id}-title" class="title">{title}</h2>
		{#if description}<p id="{id}-description" class="description">{description}</p>{/if}
		{@render children?.()}
	</div>
	{#if actions}
		<div class="actions">{@render actions()}</div>
	{/if}
</div>

<style>
	.panel {
		display: flex;
		width: 420px;
		max-width: 100%;
		flex-direction: column;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface);
		box-shadow: var(--md-sys-elevation-shadow-level3);
	}
	.panel--wide {
		width: 440px;
	}
	.body {
		display: flex;
		flex-direction: column;
		gap: 16px;
		padding: 24px 24px 0;
	}
	.title {
		margin: 0;
		font: var(--md-sys-typescale-headline-small-weight) var(--md-sys-typescale-headline-small-size) / var(--md-sys-typescale-headline-small-line-height) var(--md-sys-typescale-headline-small-font);
	}
	.description {
		margin: 0;
		font: var(--md-sys-typescale-body-medium-weight) var(--md-sys-typescale-body-medium-size) / var(--md-sys-typescale-body-medium-line-height) var(--md-sys-typescale-body-medium-font);
		color: var(--md-sys-color-on-surface-variant);
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: flex-end;
		gap: 8px;
		padding: 24px;
	}
</style>
