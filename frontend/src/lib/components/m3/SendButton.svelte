<script lang="ts">
	import type { HTMLButtonAttributes } from 'svelte/elements';
	import AgentShape from './AgentShape.svelte';
	import LoadingIndicator from './LoadingIndicator.svelte';

	// Nomi's send action: the brand cookie shape in the glow gradient, with an arrow on top.
	// While a turn is in flight it becomes the morphing loading indicator instead.

	let {
		working = false,
		size = 52,
		label = 'Send',
		...rest
	}: { working?: boolean; size?: number; label?: string } & HTMLButtonAttributes = $props();
</script>

<button
	type="submit"
	{...rest}
	class="nomi-send"
	style="--size: {size}px"
	aria-label={working ? 'Sending' : label}
	disabled={working || rest.disabled}
>
	{#if working}
		<LoadingIndicator size={size - 8} label="Sending" />
	{:else}
		<AgentShape shape="cookie9" tone="glow" size={size} class="nomi-send__shape" />
		<svg class="nomi-send__arrow" width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 19V5M5 12l7-7 7 7" /></svg>
	{/if}
</button>

<style>
	.nomi-send {
		position: relative;
		flex: none;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: var(--size);
		height: var(--size);
		padding: 0;
		border: none;
		background: transparent;
		color: var(--nomi-on-gradient-glow);
		cursor: pointer;
		transition: scale var(--nomi-motion-spatial-fast);
	}
	.nomi-send:not(:disabled):hover {
		scale: 1.06;
	}
	.nomi-send:not(:disabled):active {
		scale: 0.92;
	}
	.nomi-send:disabled {
		cursor: default;
	}
	.nomi-send :global(.nomi-send__shape) {
		position: absolute;
		inset: 0;
		transition: rotate var(--nomi-motion-spatial-slow);
	}
	.nomi-send:not(:disabled):hover :global(.nomi-send__shape) {
		rotate: 40deg;
	}
	.nomi-send__arrow {
		position: relative;
	}
</style>
