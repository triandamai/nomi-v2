<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';

	let {
		label,
		value = $bindable(''),
		error = false,
		supportingText,
		class: extraClass = '',
		...rest
	}: {
		label: string;
		value?: string;
		error?: boolean;
		supportingText?: string;
		class?: string;
	} & HTMLInputAttributes = $props();
</script>

<label class="m3-field {extraClass}">
	<span class="m3-field__label">{label}</span>
	<input class="m3-field__input" class:m3-field__input--error={error} bind:value {...rest} />
	{#if supportingText}
		<span class="m3-field__supporting" class:m3-field__supporting--error={error}>{supportingText}</span>
	{/if}
</label>

<style>
	.m3-field {
		display: flex;
		flex-direction: column;
		gap: 4px;
	}

	.m3-field__label {
		font-family: var(--md-sys-typescale-body-small-font);
		font-weight: var(--md-sys-typescale-body-small-weight);
		font-size: var(--md-sys-typescale-body-small-size);
		letter-spacing: var(--md-sys-typescale-body-small-tracking);
		color: var(--md-sys-color-on-surface-variant);
	}

	.m3-field__input {
		width: 100%;
		box-sizing: border-box;
		height: 44px;
		padding: 0 16px;
		border-radius: var(--md-sys-shape-corner-small);
		border: 1px solid var(--md-sys-color-outline);
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-body-large-font);
		font-weight: var(--md-sys-typescale-body-large-weight);
		font-size: var(--md-sys-typescale-body-large-size);
		letter-spacing: var(--md-sys-typescale-body-large-tracking);
		transition: border-color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
	}

	.m3-field__input:focus {
		outline: none;
		border: 2px solid var(--md-sys-color-primary);
		padding: 0 15px;
	}

	.m3-field__input--error {
		border-color: var(--md-sys-color-error);
	}
	.m3-field__input--error:focus {
		border-color: var(--md-sys-color-error);
	}

	.m3-field__supporting {
		font-family: var(--md-sys-typescale-body-small-font);
		font-size: var(--md-sys-typescale-body-small-size);
		letter-spacing: var(--md-sys-typescale-body-small-tracking);
		color: var(--md-sys-color-on-surface-variant);
	}
	.m3-field__supporting--error {
		color: var(--md-sys-color-error);
	}
</style>
