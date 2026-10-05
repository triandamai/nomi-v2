<script lang="ts">
	let {
		checked = $bindable(false),
		name,
		disabled = false,
		id,
		'aria-label': ariaLabel,
		onchange,
	}: {
		checked?: boolean;
		/** When set, a hidden input carries "true"/"false" so the switch submits with its form. */
		name?: string;
		disabled?: boolean;
		id?: string;
		'aria-label'?: string;
		onchange?: (checked: boolean) => void;
	} = $props();

	function toggle() {
		checked = !checked;
		onchange?.(checked);
	}
</script>

<button
	type="button"
	role="switch"
	{id}
	aria-checked={checked}
	aria-label={ariaLabel}
	{disabled}
	class="m3-switch"
	class:m3-switch--on={checked}
	onclick={toggle}
>
	<span class="m3-switch__thumb"></span>
</button>
{#if name}
	<input type="hidden" {name} value={checked ? 'true' : 'false'} />
{/if}

<style>
	.m3-switch {
		position: relative;
		flex: none;
		width: 56px;
		height: 32px;
		padding: 0;
		border-radius: var(--md-sys-shape-corner-full);
		border: 2px solid var(--md-sys-color-outline);
		background: var(--md-sys-color-surface-container-highest);
		cursor: pointer;
		transition:
			background-color var(--nomi-motion-effects-default),
			border-color var(--nomi-motion-effects-default);
	}
	.m3-switch:disabled {
		opacity: 0.38;
		cursor: not-allowed;
	}
	.m3-switch__thumb {
		position: absolute;
		top: 50%;
		left: 6px;
		width: 16px;
		height: 16px;
		translate: 0 -50%;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-outline);
		transition:
			left var(--nomi-motion-spatial-fast),
			width var(--nomi-motion-spatial-fast),
			height var(--nomi-motion-spatial-fast),
			background-color var(--nomi-motion-effects-default);
	}
	.m3-switch:active .m3-switch__thumb {
		width: 28px;
		height: 28px;
		left: 0;
	}
	.m3-switch--on {
		border-color: var(--md-sys-color-primary);
		background: var(--md-sys-color-primary);
	}
	.m3-switch--on .m3-switch__thumb {
		left: 26px;
		width: 24px;
		height: 24px;
		background: var(--md-sys-color-on-primary);
	}
	.m3-switch--on:active .m3-switch__thumb {
		left: 22px;
		width: 28px;
		height: 28px;
	}
</style>
