<script lang="ts">
	import { enhance } from '$app/forms';
	import { LANGUAGES, type Locale } from '$lib/i18n';
	import { m } from '$lib/paraglide/messages';
	import { getLocale, setLocale } from '$lib/paraglide/runtime';

	// The app's language, and the language the crew replies in. Saved to the account, then the
	// page reloads in the new language.
	let current = $state<Locale>(getLocale());
	let saving = $state(false);
</script>

<form
	method="POST"
	action="/account?/updateLanguage"
	class="languages"
	use:enhance={() => {
		saving = true;
		return async ({ result, update }) => {
			saving = false;
			if (result.type === 'success') {
				setLocale(current);
			} else {
				current = getLocale();
				await update({ reset: false });
			}
		};
	}}
>
	{#each LANGUAGES as option (option.value)}
		{@const selected = current === option.value}
		<button
			type="submit"
			name="language"
			value={option.value}
			class="language"
			class:language--selected={selected}
			aria-pressed={selected}
			disabled={saving}
			lang={option.value}
			onclick={() => (current = option.value)}
		>
			<span class="language__code" aria-hidden="true">{option.value.toUpperCase()}</span>
			<span class="language__text">
				<span class="language__name">{option.label}</span>
				<span class="language__hint">{option.hint}</span>
			</span>
			{#if selected}
				<svg class="language__check" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m5 12 5 5 9-10" /></svg>
			{/if}
		</button>
	{/each}
</form>
<p class="languages__hint">{m.language_hint()}</p>

<style>
	.languages {
		display: flex;
		flex-direction: column;
		gap: 2px;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-extra-large);
	}
	.language {
		display: flex;
		align-items: center;
		gap: 14px;
		min-height: 64px;
		padding: 10px 16px;
		border: none;
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		text-align: left;
		cursor: pointer;
		transition: background-color var(--nomi-motion-effects-fast);
	}
	.language:hover:not(:disabled) {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 6%, var(--md-sys-color-surface-container-lowest));
	}
	.language:disabled {
		cursor: progress;
	}
	.language--selected {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.language--selected:hover:not(:disabled) {
		background: var(--md-sys-color-secondary-container);
	}
	.language__code {
		display: grid;
		flex-shrink: 0;
		place-items: center;
		width: 40px;
		height: 40px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-primary);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		font-weight: 500;
		transition: border-radius var(--nomi-motion-spatial-fast);
	}
	.language--selected .language__code {
		border-radius: 50%;
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
	.language__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
	}
	.language__name {
		font-size: 1rem;
		font-weight: 600;
	}
	.language__hint {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.language--selected .language__hint {
		color: inherit;
		opacity: 0.8;
	}
	.language__check {
		flex-shrink: 0;
		color: var(--md-sys-color-primary);
	}
	.languages__hint {
		margin: 8px 8px 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		line-height: 1.5;
	}
</style>
