<script lang="ts">
	// A publish's or a step's state as a small rounded square: what worked, what broke, what's
	// still going, what never ran. The words go next to it or in `label`; colour isn't the only
	// signal (each state has its own mark).
	import IconAlert from '$lib/components/icons/IconAlert.svelte';
	import IconCheck from '$lib/components/icons/IconCheck.svelte';
	import IconMoreHoriz from '$lib/components/icons/IconMoreHoriz.svelte';

	let { status, size = 32, label }: { status: string; size?: number; label?: string } = $props();

	const tone = $derived(
		status === 'succeeded' ? 'ok' : status === 'failed' ? 'failed' : status === 'running' ? 'running' : 'idle',
	);
	const icon = $derived(Math.round(size * 0.55));
</script>

<span class="square square--{tone}" style:--size="{size}px" role={label ? 'img' : undefined} aria-label={label} aria-hidden={label ? undefined : 'true'}>
	{#if tone === 'ok'}<IconCheck size={icon} />{:else if tone === 'failed'}<IconAlert size={icon} />{:else if tone === 'running'}<IconMoreHoriz size={icon} />{:else}<span class="dash"></span>{/if}
</span>

<style>
	.square {
		display: inline-grid;
		flex: none;
		width: var(--size);
		height: var(--size);
		place-items: center;
		border-radius: calc(var(--size) * 0.3);
	}
	.square--ok {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.square--failed {
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}
	.square--running {
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}
	.square--idle {
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
	}
	.dash {
		width: 40%;
		height: 2px;
		border-radius: 1px;
		background: currentColor;
	}
</style>
