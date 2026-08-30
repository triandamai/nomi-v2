<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';

	let {
		group = $bindable(''),
		value,
		name,
		label,
		disabled = false,
		class: extraClass = '',
		...rest
	}: {
		group?: string;
		value: string;
		name: string;
		label?: string;
		disabled?: boolean;
		class?: string;
	} & Omit<HTMLInputAttributes, 'type' | 'checked' | 'value' | 'name'> = $props();
</script>

<label class="m3-radio {extraClass}" class:m3-radio--disabled={disabled}>
	<span class="m3-radio__circle">
		<input type="radio" {name} {value} bind:group {disabled} {...rest} />
		<span class="m3-radio__indicator" class:m3-radio__indicator--checked={group === value}></span>
	</span>
	{#if label}
		<span class="m3-radio__label">{label}</span>
	{/if}
</label>

<style>
	.m3-radio {
		display: inline-flex;
		align-items: center;
		gap: 8px;
		cursor: pointer;
	}
	.m3-radio--disabled {
		cursor: not-allowed;
		opacity: 0.38;
	}

	.m3-radio__circle {
		position: relative;
		display: inline-flex;
		width: 20px;
		height: 20px;
	}
	.m3-radio__circle input {
		position: absolute;
		inset: -8px;
		margin: 0;
		opacity: 0;
		cursor: pointer;
	}
	.m3-radio--disabled .m3-radio__circle input {
		cursor: not-allowed;
	}

	.m3-radio__indicator {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 20px;
		height: 20px;
		border-radius: var(--md-sys-shape-corner-full);
		border: 2px solid var(--md-sys-color-outline);
		transition: border-color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
	}
	.m3-radio__indicator::after {
		content: '';
		width: 10px;
		height: 10px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-primary);
		transform: scale(0);
		transition: transform var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
	}
	.m3-radio__indicator--checked {
		border-color: var(--md-sys-color-primary);
	}
	.m3-radio__indicator--checked::after {
		transform: scale(1);
	}
	.m3-radio__circle input:focus-visible ~ .m3-radio__indicator {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}

	.m3-radio__label {
		font-family: var(--md-sys-typescale-body-large-font);
		font-size: var(--md-sys-typescale-body-large-size);
		color: var(--md-sys-color-on-surface);
	}
</style>
