<!-- frontend/src/lib/components/blocks/FileDeleteBlock.svelte -->
<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'file_delete' }> } = $props();
</script>

<div class="m3-block-card m3-block-card--delete">
	<svg class="m3-block-card__icon" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M4 7h16M10 11v6M14 11v6M6 7l1 12a2 2 0 0 0 2 2h6a2 2 0 0 0 2-2l1-12M9 7V4h6v3" /></svg>
	<span class="md-body-medium">{m.block_deleted()} <code>{block.path}</code></span>
</div>

<style>
	.m3-block-card {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 16px;
		border-radius: var(--md-sys-shape-corner-large-increased);
	}
	.m3-block-card--delete {
		background: color-mix(in srgb, var(--md-sys-color-error) 6%, var(--md-sys-color-surface-container-low));
	}
	.m3-block-card__icon {
		flex: none;
		color: var(--md-sys-color-error);
	}
</style>
