<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import type { HTMLButtonAttributes } from 'svelte/elements';
	import AgentShape from './AgentShape.svelte';
	import LoadingIndicator from './LoadingIndicator.svelte';

	// Nomi's send action: the brand cookie shape in the glow gradient, with an arrow on top.
	// While a turn is in flight it becomes the morphing loading indicator instead; with `stopForm`
	// set, that indicator carries a stop square and submits the given form (see ChatThread).

	let {
		working = false,
		size = 52,
		label = m.send_send(),
		stopForm,
		...rest
	}: {
		working?: boolean;
		size?: number;
		label?: string;
		/** id of the form that stops the crew; makes the in-flight state a Stop button. */
		stopForm?: string;
	} & HTMLButtonAttributes = $props();

	const stoppable = $derived(working && stopForm !== undefined);
</script>

<button
	type="submit"
	{...rest}
	form={stoppable ? stopForm : rest.form}
	class="nomi-send"
	class:nomi-send--stop={stoppable}
	style="--size: {size}px"
	aria-label={stoppable ? m.send_stop_crew() : working ? m.send_sending() : label}
	title={stoppable ? m.send_stop() : undefined}
	disabled={stoppable ? false : working || rest.disabled}
>
	{#if working}
		<LoadingIndicator size={size - 8} label={stoppable ? m.send_working() : m.send_sending()} />
		{#if stoppable}
			<svg class="nomi-send__stop" width="16" height="16" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true"><rect x="5" y="5" width="14" height="14" rx="3.5" /></svg>
		{/if}
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
	/* In-flight Stop: the loader keeps morphing, a solid square sits in its middle. */
	.nomi-send__stop {
		position: absolute;
		color: var(--md-sys-color-on-surface);
		transition: scale var(--nomi-motion-spatial-fast);
	}
	.nomi-send--stop:hover .nomi-send__stop {
		scale: 1.15;
	}
</style>
