<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Something to ask, with who it would go to ("→ dana").
	type Props = { agent: CrewId; label: string; route?: string; onclick?: () => void };
	let { agent, label, route, onclick }: Props = $props();
</script>

<button type="button" {onclick} class="suggestion">
	<Mascot {agent} size={20} />
	<span class="label">{label}</span>
	{#if route}<span class="route">{route}</span>{/if}
</button>

<style>
	.suggestion {
		display: flex;
		width: 100%;
		min-height: 48px;
		align-items: center;
		gap: 12px;
		padding: 8px 16px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		background: none;
		color: var(--md-sys-color-on-surface);
		text-align: left;
		cursor: pointer;
		font: var(--md-sys-typescale-body-medium-weight) var(--md-sys-typescale-body-medium-size) / var(--md-sys-typescale-body-medium-line-height) var(--md-sys-typescale-body-medium-font);
	}
	.suggestion:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.suggestion:focus-visible {
		outline: 3px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.label {
		min-width: 0;
		flex: 1;
	}
	.route {
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
	}
</style>
