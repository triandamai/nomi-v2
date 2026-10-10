<script lang="ts">
	import type { Snippet } from 'svelte';

	// The field a date or time picker sits behind: it looks like a text field, shows what's
	// picked (or a hint), and opens the picker when pressed.
	let {
		id,
		label,
		text,
		placeholder = '',
		error = false,
		disabled = false,
		supportingText,
		icon,
		onclick,
	}: {
		id?: string;
		label: string;
		/** What's picked, written out; empty shows the placeholder. */
		text: string;
		placeholder?: string;
		error?: boolean;
		disabled?: boolean;
		supportingText?: string;
		icon: Snippet;
		onclick: () => void;
	} = $props();
</script>

<div class="picker-field">
	<span class="picker-field__label" id="{id}-label">{label}</span>
	<button
		{id}
		type="button"
		class="picker-field__box"
		class:picker-field__box--error={error}
		class:picker-field__box--empty={!text}
		aria-haspopup="dialog"
		aria-labelledby="{id}-label {id}"
		{disabled}
		{onclick}
	>
		<span class="picker-field__text">{text || placeholder}</span>
		<span class="picker-field__icon" aria-hidden="true">{@render icon()}</span>
	</button>
	{#if supportingText}
		<span class="picker-field__supporting" class:picker-field__supporting--error={error}>{supportingText}</span>
	{/if}
</div>

<style>
	.picker-field {
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
	}
	.picker-field__label,
	.picker-field__supporting {
		font-family: var(--md-sys-typescale-body-small-font);
		font-size: var(--md-sys-typescale-body-small-size);
		letter-spacing: var(--md-sys-typescale-body-small-tracking);
		color: var(--md-sys-color-on-surface-variant);
	}
	.picker-field__supporting--error {
		color: var(--md-sys-color-error);
	}
	.picker-field__box {
		display: flex;
		align-items: center;
		gap: 12px;
		width: 100%;
		height: 52px;
		padding: 0 12px 0 16px;
		box-sizing: border-box;
		border: 1.5px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-body-large-font);
		font-size: var(--md-sys-typescale-body-large-size);
		letter-spacing: var(--md-sys-typescale-body-large-tracking);
		text-align: left;
		cursor: pointer;
		transition:
			border-color var(--nomi-motion-effects-fast),
			background-color var(--nomi-motion-effects-fast);
	}
	.picker-field__box:hover:not(:disabled) {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 4%, var(--md-sys-color-surface));
	}
	.picker-field__box:focus-visible {
		outline: none;
		border: 2px solid var(--md-sys-color-primary);
		padding: 0 11.5px 0 15.5px;
	}
	.picker-field__box:disabled {
		opacity: 0.38;
		cursor: default;
	}
	.picker-field__box--error {
		border-color: var(--md-sys-color-error);
	}
	.picker-field__box--empty .picker-field__text {
		color: var(--md-sys-color-on-surface-variant);
	}
	.picker-field__text {
		flex: 1;
		min-width: 0;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.picker-field__icon {
		display: inline-flex;
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
