<script lang="ts">
	import { page } from '$app/state';
	import Button from '$lib/components/m3/Button.svelte';
	import LanguagePicker from '$lib/components/LanguagePicker.svelte';
	import { m } from '$lib/paraglide/messages';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const methods = $derived(data.methods);
	const googleError = page.url.searchParams.get('google_error');
	const justLinked = page.url.searchParams.get('google') === 'linked';
</script>

<div class="account">
	<div class="account__inner">
		<h1 class="md-display-small account__title">{m.account_title()}</h1>

		<section class="section">
			<h2 class="section__label">{m.account_email()}</h2>
			<p class="md-body-large account__email">{data.profile?.email ?? '—'}</p>
		</section>

		<section class="section" aria-labelledby="language-title">
			<h2 id="language-title" class="section__label">{m.account_language()}</h2>
			<LanguagePicker />
		</section>

		<section class="section" aria-labelledby="methods-title">
			<h2 id="methods-title" class="section__label">{m.account_sign_in_title()}</h2>
			{#if justLinked}
				<p class="note note--ok" role="status">{m.account_google_linked()}</p>
			{/if}
			{#if googleError || form?.error}
				<p class="note note--error" role="alert">{form?.error ?? (googleError === 'cancelled' ? m.account_google_cancelled() : googleError)}</p>
			{/if}
			<ul class="rows">
				<li class="row">
					<span class="row__icon" aria-hidden="true">
						<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="4" y="10" width="16" height="11" rx="2" /><path d="M8 10V7a4 4 0 0 1 8 0v3" /></svg>
					</span>
					<span class="row__text">
						<span class="row__headline">{m.account_password_method()}</span>
						<span class="row__supporting">{methods?.password ? m.account_on() : m.account_password_not_set()}</span>
					</span>
				</li>
				<li class="row">
					<span class="row__icon" aria-hidden="true">
						<svg width="18" height="18" viewBox="0 0 48 48"><path fill="#EA4335" d="M24 9.5c3.54 0 6.71 1.22 9.21 3.6l6.85-6.85C35.9 2.38 30.47 0 24 0 14.62 0 6.51 5.38 2.56 13.22l7.98 6.19C12.43 13.72 17.74 9.5 24 9.5z" /><path fill="#4285F4" d="M46.98 24.55c0-1.57-.15-3.09-.38-4.55H24v9.02h12.94c-.58 2.96-2.26 5.48-4.78 7.18l7.73 6c4.51-4.18 7.09-10.36 7.09-17.65z" /><path fill="#FBBC05" d="M10.53 28.59c-.48-1.45-.76-2.99-.76-4.59s.27-3.14.76-4.59l-7.98-6.19C.92 16.46 0 20.12 0 24c0 3.88.92 7.54 2.56 10.78l7.97-6.19z" /><path fill="#34A853" d="M24 48c6.48 0 11.93-2.13 15.89-5.81l-7.73-6c-2.15 1.45-4.92 2.3-8.16 2.3-6.26 0-11.57-4.22-13.47-9.91l-7.98 6.19C6.51 42.62 14.62 48 24 48z" /></svg>
					</span>
					<span class="row__text">
						<span class="row__headline">Google</span>
						<span class="row__supporting">
							{#if methods?.google_email}{methods.google_email}{:else if methods?.configured}{m.account_not_linked()}{:else}{m.account_google_unavailable()}{/if}
						</span>
					</span>
					{#if methods?.google_email}
						{#if methods.password}
							<form method="POST" action="?/unlink"><Button variant="text" size="s" type="submit">{m.account_unlink()}</Button></form>
						{/if}
					{:else if methods?.configured}
						<form method="POST" action="?/link"><Button variant="tonal" size="s" type="submit">{m.account_link_google()}</Button></form>
					{/if}
				</li>
			</ul>
			<p class="hint">
				{m.account_google_hint()} <a href="/connections">{m.nav_connections()}</a>.
			</p>
		</section>
	</div>
</div>

<style>
	.account {
		height: 100%;
		overflow-y: auto;
		padding: 32px clamp(16px, 4vw, 56px) 56px;
		box-sizing: border-box;
	}
	.account__inner {
		display: flex;
		flex-direction: column;
		gap: 28px;
		max-width: 640px;
	}
	.account__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.account__email {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.section {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.section__label {
		margin: 0 8px;
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		font-weight: 400;
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	.rows {
		display: flex;
		flex-direction: column;
		gap: 2px;
		margin: 0;
		padding: 0;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-extra-large);
		list-style: none;
	}
	.row {
		display: flex;
		align-items: center;
		gap: 14px;
		min-height: 64px;
		padding: 10px 12px 10px 16px;
		background: var(--md-sys-color-surface-container-lowest);
	}
	.row__icon {
		display: flex;
		flex-shrink: 0;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-primary);
	}
	.row__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.row__headline {
		color: var(--md-sys-color-on-surface);
		font-size: 1rem;
		font-weight: 600;
	}
	.row__supporting {
		overflow-wrap: anywhere;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.note {
		margin: 0;
		padding: 12px 16px;
		border-radius: var(--md-sys-shape-corner-large);
		font-size: 0.875rem;
	}
	.note--ok {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.note--error {
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}
	.hint {
		margin: 0 8px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		line-height: 1.5;
	}
	.hint a {
		color: var(--md-sys-color-primary);
		font-weight: 600;
	}
</style>
