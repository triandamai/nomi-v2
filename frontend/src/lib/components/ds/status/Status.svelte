<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// A dot plus a word, so colour never carries meaning alone. Who is working shows their shape
	// instead of a dot. States, not decoration: no pulsing. `small` sets the word smaller.
	type Props = { kind: 'live' | 'ready' | 'idle' | 'failed' | 'attention' | 'working'; label: string; agent?: CrewId; small?: boolean };
	let { kind, label, agent, small = false }: Props = $props();
</script>

<span class="status status--{kind}" class:status--small={small}>
	{#if kind === 'working' && agent}
		<Mascot {agent} size={20} />
	{:else}
		<span data-dot class="dot"></span>
	{/if}
	<span class="label">{label}</span>
</span>

<style>
	.status {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		color: var(--md-sys-color-on-surface);
	}
	.dot {
		width: 8px;
		height: 8px;
		flex-shrink: 0;
		border-radius: var(--md-sys-shape-corner-full);
		background: currentColor;
	}
	.label {
		font: var(--md-sys-typescale-label-large-weight) var(--md-sys-typescale-label-large-size) / var(--md-sys-typescale-label-large-line-height) var(--md-sys-typescale-label-large-font);
	}
	.status--small .label,
	.status--ready .label,
	.status--idle .label {
		font-size: var(--md-sys-typescale-label-small-size);
	}
	.status--live .dot,
	.status--ready .dot {
		background: var(--md-sys-color-primary);
	}
	.status--ready .label,
	.status--idle {
		color: var(--md-sys-color-on-surface-variant);
	}
	.status--failed {
		color: var(--md-sys-color-error);
	}
	.status--attention {
		color: var(--md-sys-color-tertiary);
	}
</style>
