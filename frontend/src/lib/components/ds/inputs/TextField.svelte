<script lang="ts">
	import { CircleAlert } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Twin of frontend-new's TextField, drawn as the M3 outlined field: label · field · helper. An
	// error swaps the outline and takes the helper's place to say what to do next.
	type Props = {
		label: string;
		value?: string;
		helper?: string;
		/** The helper at 11px, where the pen sets it smaller (the first-run name). */
		smallHelper?: boolean;
		error?: string;
		placeholder?: string;
		/** For machine values: slugs, keys, URLs. */
		mono?: boolean;
		disabled?: boolean;
		required?: boolean;
		type?: 'text' | 'email' | 'url' | 'password';
		name?: string;
		autocomplete?: HTMLInputElement['autocomplete'];
		oninput?: (value: string) => void;
		/** Shows the focus state without focus (the gallery). */
		'data-focus'?: boolean;
	};
	let {
		label,
		value = $bindable(''),
		helper,
		smallHelper = false,
		error,
		placeholder,
		mono = false,
		disabled = false,
		required = false,
		type = 'text',
		name,
		autocomplete,
		oninput,
		'data-focus': showFocus = false,
	}: Props = $props();

	const id = $props.id();
	const note = $derived(error ?? helper);
</script>

<div class="field" class:field--disabled={disabled}>
	<label for={id} class="label">{label}</label>
	<div class="box" class:box--error={error} data-focus={showFocus || undefined}>
		<input
			{id}
			{type}
			{name}
			{placeholder}
			{disabled}
			{required}
			{autocomplete}
			bind:value
			oninput={() => oninput?.(value)}
			aria-invalid={error ? 'true' : undefined}
			aria-describedby={note ? `${id}-note` : undefined}
			class:mono
		/>
		{#if error}<span class="error-icon"><Icon icon={CircleAlert} size={20} /></span>{/if}
	</div>
	{#if note}
		<p id="{id}-note" class="note" class:note--small={smallHelper && !error} class:note--error={error}>{note}</p>
	{/if}
</div>

<style>
	.field {
		display: flex;
		width: 100%;
		flex-direction: column;
		gap: 4px;
	}
	.field--disabled {
		opacity: 0.38;
	}
	.label {
		font: var(--md-sys-typescale-body-small-weight) var(--md-sys-typescale-body-small-size) / var(--md-sys-typescale-body-small-line-height) var(--md-sys-typescale-body-small-font);
		color: var(--md-sys-color-on-surface-variant);
	}
	.box {
		display: flex;
		height: 48px;
		align-items: center;
		gap: 8px;
		padding: 0 14px;
		border: 1.5px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface);
	}
	.box:focus-within,
	.box[data-focus] {
		border: 2px solid var(--md-sys-color-primary);
		padding: 0 13.5px;
	}
	.box--error,
	.box--error:focus-within {
		border-color: var(--md-sys-color-error);
	}
	input {
		min-width: 0;
		flex: 1;
		border: 0;
		background: transparent;
		outline: none;
		color: var(--md-sys-color-on-surface);
		caret-color: var(--md-sys-color-primary);
		font: var(--md-sys-typescale-body-large-weight) var(--md-sys-typescale-body-large-size) / var(--md-sys-typescale-body-large-line-height) var(--md-sys-typescale-body-large-font);
	}
	input.mono {
		font-family: var(--md-ref-typeface-mono);
	}
	input::placeholder {
		color: var(--md-sys-color-on-surface-variant);
	}
	.error-icon {
		display: inline-flex;
		color: var(--md-sys-color-error);
	}
	.note {
		margin: 0;
		font: var(--md-sys-typescale-body-small-weight) var(--md-sys-typescale-body-small-size) / var(--md-sys-typescale-body-small-line-height) var(--md-sys-typescale-body-small-font);
		color: var(--md-sys-color-on-surface-variant);
	}
	.note--small {
		font-size: var(--md-sys-typescale-label-small-size);
	}
	.note--error {
		color: var(--md-sys-color-error);
	}
</style>
