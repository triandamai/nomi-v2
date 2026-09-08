<!-- frontend/src/lib/components/blocks/FileDeleteBlock.svelte -->
<script lang="ts">
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'file_delete' }> } = $props();
</script>

<div class="m3-block-card m3-block-card--delete">
	<span class="m3-block-card__icon" aria-hidden="true">🗑️</span>
	<span class="md-body-medium">Deleted <code>{block.path}</code></span>
</div>

<style>
	.m3-block-card {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 14px;
		border-radius: var(--md-sys-shape-corner-medium);
		border: 1px solid var(--md-sys-color-outline-variant);
	}
	.m3-block-card--delete {
		background: color-mix(in srgb, var(--md-sys-color-error) 6%, var(--md-sys-color-surface-container-low));
	}
	.m3-block-card__icon {
		font-size: 1.1em;
	}
</style>
