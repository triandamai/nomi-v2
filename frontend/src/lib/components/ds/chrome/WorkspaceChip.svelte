<script lang="ts">
	import { ChevronsUpDown } from '@lucide/svelte';

	/** Which workspace's projects the Build group shows ("T Trian's space ↕"); opens the switcher. */
	let { name, onclick }: { name: string; onclick?: () => void } = $props();
</script>

<button type="button" {onclick} aria-haspopup="menu" class="chip">
	<span aria-hidden="true" class="initial">{name.charAt(0).toUpperCase()}</span>
	<span class="name">{name}</span>
	<ChevronsUpDown size={14} strokeWidth={1.5} aria-hidden="true" />
</button>

<style>
	.chip {
		display: inline-flex;
		height: 28px;
		align-items: center;
		gap: 6px;
		padding: 0 8px 0 4px;
		border: 0;
		border-radius: var(--md-sys-shape-corner-small);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
		cursor: pointer;
	}
	.chip:hover {
		background: var(--md-sys-color-surface-container-highest);
	}
	.chip:focus-visible {
		outline: 3px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.initial {
		display: inline-flex;
		width: 20px;
		height: 20px;
		align-items: center;
		justify-content: center;
		border-radius: var(--md-sys-shape-corner-extra-small);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font: var(--md-sys-typescale-label-medium-weight) var(--md-sys-typescale-label-medium-size) / var(--md-sys-typescale-label-medium-line-height) var(--md-sys-typescale-label-medium-font);
	}
	.name {
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
	}
</style>
