<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';
	import IconCheck from '../icons/IconCheck.svelte';

	let {
		checked = $bindable(false),
		label,
		disabled = false,
		class: extraClass = '',
		...rest
	}: {
		checked?: boolean;
		label?: string;
		disabled?: boolean;
		class?: string;
	} & Omit<HTMLInputAttributes, 'type' | 'checked'> = $props();
</script>

<label class="m3-checkbox {extraClass}" class:m3-checkbox--disabled={disabled}>
	<span class="m3-checkbox__box">
		<input type="checkbox" bind:checked {disabled} {...rest} />
		<span class="m3-checkbox__indicator" class:m3-checkbox__indicator--checked={checked}>
			{#if checked}
				<IconCheck size={14} />
			{/if}
		</span>
	</span>
	{#if label}
		<span class="m3-checkbox__label">{label}</span>
	{/if}
</label>

<style>
	.m3-checkbox {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		cursor: pointer;
	}
	.m3-checkbox--disabled {
		cursor: not-allowed;
		opacity: 0.38;
	}

	.m3-checkbox__box {
		position: relative;
		display: inline-flex;
		width: 18px;
		height: 18px;
	}
	.m3-checkbox__box input {
		position: absolute;
		inset: -8px;
		margin: 0;
		opacity: 0;
		cursor: pointer;
	}
	.m3-checkbox--disabled .m3-checkbox__box input {
		cursor: not-allowed;
	}

	.m3-checkbox__indicator {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 18px;
		height: 18px;
		border-radius: var(--md-sys-shape-corner-extra-small);
		border: 2px solid var(--md-sys-color-outline);
		color: var(--md-sys-color-on-primary);
		background: transparent;
		transition:
			background-color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard),
			border-color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
	}
	.m3-checkbox__indicator--checked {
		background: var(--md-sys-color-primary);
		border-color: var(--md-sys-color-primary);
	}
	.m3-checkbox__box input:focus-visible ~ .m3-checkbox__indicator {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}

	.m3-checkbox__label {
		font-family: var(--md-sys-typescale-body-large-font);
		font-size: var(--md-sys-typescale-body-large-size);
		color: var(--md-sys-color-on-surface);
	}
</style>
