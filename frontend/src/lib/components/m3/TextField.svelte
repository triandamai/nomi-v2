<script lang="ts">
	import type { HTMLInputAttributes } from 'svelte/elements';
	import { Eye, EyeOff } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';

	let {
		label,
		value = $bindable(''),
		error = false,
		supportingText,
		type = 'text',
		class: extraClass = '',
		...rest
	}: {
		label: string;
		value?: string;
		error?: boolean;
		supportingText?: string;
		class?: string;
	} & HTMLInputAttributes = $props();

	// A secret (type="password") gets an eye to show it while checking what was typed.
	const secret = $derived(type === 'password');
	let revealed = $state(false);
</script>

<label class="m3-field {extraClass}">
	<span class="m3-field__label">{label}</span>
	<span class="m3-field__control">
		<input
			class="m3-field__input"
			class:m3-field__input--error={error}
			class:m3-field__input--secret={secret}
			type={secret && revealed ? 'text' : type}
			aria-invalid={error ? 'true' : undefined}
			bind:value
			{...rest}
		/>
		{#if secret}
			<button
				type="button"
				class="m3-field__eye"
				aria-label={revealed ? m.field_hide_secret() : m.field_show_secret()}
				aria-pressed={revealed}
				onclick={() => (revealed = !revealed)}
			>
				{#if revealed}<EyeOff size={20} aria-hidden="true" />{:else}<Eye size={20} aria-hidden="true" />{/if}
			</button>
		{/if}
	</span>
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

	.m3-field__control {
		position: relative;
		display: block;
	}

	.m3-field__input--secret {
		padding-right: 48px;
	}
	.m3-field__input--secret:focus {
		padding-right: 47px;
	}

	.m3-field__eye {
		position: absolute;
		top: 0;
		right: 0;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 48px;
		height: 48px;
		padding: 0;
		border: 0;
		border-radius: var(--md-sys-shape-corner-large);
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		cursor: pointer;
	}
	.m3-field__eye:hover {
		color: var(--md-sys-color-on-surface);
	}
	.m3-field__eye:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: -4px;
	}

	.m3-field__input {
		width: 100%;
		box-sizing: border-box;
		height: 48px;
		padding: 0 14px;
		border-radius: var(--md-sys-shape-corner-large);
		border: 1.5px solid var(--md-sys-color-outline);
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
		padding: 0 15.5px;
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
