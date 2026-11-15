<script lang="ts">
	import { page } from '$app/state';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import { enhance } from '$app/forms';
	import LanguagePicker from '$lib/components/LanguagePicker.svelte';
	import DeviceList from './DeviceList.svelte';
	import ShareList from './ShareList.svelte';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import Switch from '$lib/components/m3/Switch.svelte';
	import Checkbox from '$lib/components/m3/Checkbox.svelte';
	import { FORMAT_REGIONS, formatSample } from '$lib/accountSettings';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const methods = $derived(data.methods);

	// Settings › General: kept as they're changed (the form saves on each change).
	let formats = $state('');
	let defaultAgent = $state('');
	let enterSends = $state(true);
	let showReplyTokens = $state(false);
	$effect(() => {
		formats = data.preferences?.formats ?? '';
		defaultAgent = data.preferences?.default_agent ?? '';
		enterSends = data.preferences?.enter_sends ?? true;
		showReplyTokens = data.preferences?.show_reply_tokens ?? false;
	});
	let generalForm: HTMLFormElement | undefined = $state();
	const saveGeneral = () => queueMicrotask(() => generalForm?.requestSubmit());
	const crew = $derived((data.crew ?? []).filter((member) => member.agent_type !== 'supervisor'));

	let changingEmail = $state(false);
	const emailChallenge = $derived(form && 'emailChallenge' in form ? (form.emailChallenge as string | undefined) : undefined);
	let deleting = $state(false);
	let deleteConfirmed = $state(false);
	const deleteChallenge = $derived(form && 'deleteChallenge' in form ? (form.deleteChallenge as string | undefined) : undefined);
	const googleError = page.url.searchParams.get('google_error');
	const justLinked = page.url.searchParams.get('google') === 'linked';
</script>

<div class="account">
	<div class="account__inner">
		<h1 class="md-display-small account__title">{m.account_title()}</h1>

		<section class="section" aria-labelledby="email-title">
			<h2 id="email-title" class="section__label">{m.account_email()}</h2>
			<div class="email-row">
				<p class="md-body-large account__email">{form && 'emailChanged' in form && form.emailChanged ? form.emailChanged : (data.profile?.email ?? '—')}</p>
				{#if !changingEmail && !emailChallenge}
					<Button variant="text" size="s" onclick={() => (changingEmail = true)}>{m.account_email_change()}</Button>
				{/if}
			</div>
			{#if form && 'emailChanged' in form && form.emailChanged}
				<p class="note note--ok" role="status">{m.account_email_changed({ email: form.emailChanged })}</p>
			{/if}
			{#if form && 'emailError' in form && form.emailError}
				<p class="note note--error" role="alert">{form.emailError}</p>
			{/if}
			{#if emailChallenge}
				<form method="POST" action="?/emailVerify" use:enhance class="password">
					<p class="hint">{m.account_email_code_sent({ email: form && 'emailSentTo' in form && form.emailSentTo ? String(form.emailSentTo) : '' })}</p>
					<input type="hidden" name="challenge_id" value={emailChallenge} />
					<TextField id="email_code" name="code" label={m.account_code()} autocomplete="one-time-code" inputmode="numeric" required />
					<div><Button type="submit" variant="tonal" size="s">{m.account_email_confirm()}</Button></div>
				</form>
			{:else if changingEmail}
				<form method="POST" action="?/emailStart" use:enhance class="password">
					<TextField id="new_email" name="email" type="email" label={m.account_email_new()} autocomplete="email" required />
					<div class="actions">
						<Button type="submit" variant="tonal" size="s">{m.account_email_send_code()}</Button>
						<Button variant="text" size="s" onclick={() => (changingEmail = false)}>{m.account_cancel()}</Button>
					</div>
				</form>
			{/if}
		</section>

		<section class="section" aria-labelledby="language-title">
			<h2 id="language-title" class="section__label">{m.account_language()}</h2>
			<LanguagePicker />
		</section>

		<section class="section" aria-labelledby="general-title">
			<h2 id="general-title" class="section__label">{m.account_general()}</h2>
			{#if form && 'generalError' in form && form.generalError}
				<p class="note note--error" role="alert">{form.generalError}</p>
			{/if}
			<form method="POST" action="?/general" bind:this={generalForm} use:enhance={() => async ({ update }) => update({ reset: false })}>
				<input type="hidden" name="enter_sends" value={String(enterSends)} />
				<input type="hidden" name="show_reply_tokens" value={String(showReplyTokens)} />
				<ul class="rows">
					<li class="row">
						<label class="row__text" for="formats">
							<span class="row__headline">{m.account_formats()}</span>
							<span class="row__supporting">{formatSample(formats, getLocale())}</span>
						</label>
						<select id="formats" name="formats" class="picker" bind:value={formats} onchange={saveGeneral}>
							{#each FORMAT_REGIONS as region (region)}
								<option value={region}>{region || m.account_formats_language()}</option>
							{/each}
						</select>
					</li>
					<li class="row">
						<label class="row__text" for="default_agent">
							<span class="row__headline">{m.account_default_agent()}</span>
							<span class="row__supporting">{m.account_default_agent_hint()}</span>
						</label>
						<select id="default_agent" name="default_agent" class="picker" bind:value={defaultAgent} onchange={saveGeneral}>
							<option value="">Nomi</option>
							{#each crew.filter((member) => member.agent_type !== 'chitchat') as member (member.agent_type)}
								<option value={member.agent_type}>{member.name}</option>
							{/each}
						</select>
					</li>
					<li class="row">
						<span class="row__text">
							<span class="row__headline">{m.account_enter_sends()}</span>
							<span class="row__supporting">{enterSends ? m.account_enter_sends_on() : m.account_enter_sends_off()}</span>
						</span>
						<Switch bind:checked={enterSends} aria-label={m.account_enter_sends()} onchange={saveGeneral} />
					</li>
					<li class="row">
						<span class="row__text">
							<span class="row__headline">{m.account_reply_tokens()}</span>
							<span class="row__supporting">{m.account_reply_tokens_hint()}</span>
						</span>
						<Switch bind:checked={showReplyTokens} aria-label={m.account_reply_tokens()} onchange={saveGeneral} />
					</li>
				</ul>
			</form>
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
			<form method="POST" action="?/password" use:enhance class="password">
				<h3 class="password__title">{methods?.password ? m.account_password_change() : m.account_password_set()}</h3>
				{#if form && 'passwordSaved' in form && form.passwordSaved}
					<p class="note note--ok" role="status">{m.account_password_saved()}</p>
				{/if}
				{#if form && 'passwordError' in form && form.passwordError}
					<p class="note note--error" role="alert">{form.passwordError}</p>
				{/if}
				{#if methods?.password}
					<TextField id="current_password" name="current_password" type="password" label={m.account_password_current()} autocomplete="current-password" required />
				{/if}
				<TextField id="new_password" name="new_password" type="password" label={m.account_password_new()} autocomplete="new-password" minlength={8} required />
				<div><Button type="submit" variant="tonal" size="s">{m.account_password_save()}</Button></div>
			</form>
			<p class="hint">
				{m.account_google_hint()} <a href="/connections">{m.nav_connections()}</a>.
			</p>
		</section>

		<section class="section" aria-labelledby="devices-title">
			<h2 id="devices-title" class="section__label">{m.account_devices()}</h2>
			{#if form && 'signedOut' in form && form.signedOut}
				<p class="note note--ok" role="status">{m.account_device_signed_out({ name: form.signedOut })}</p>
			{/if}
			{#if form && 'deviceError' in form && form.deviceError}
				<p class="note note--error" role="alert">{form.deviceError}</p>
			{/if}
			{#if data.devices}
				<DeviceList devices={data.devices} />
			{/if}
			<p class="hint">{m.account_devices_hint()}</p>
		</section>

		<section id="data" class="section" aria-labelledby="data-title">
			<h2 id="data-title" class="section__label">{m.account_data()}</h2>
			<h3 class="section__sub">{m.account_shares()}</h3>
			{#if form && 'shareRevoked' in form && form.shareRevoked}
				<p class="note note--ok" role="status">{m.account_share_revoked()}</p>
			{/if}
			{#if form && 'shareError' in form && form.shareError}
				<p class="note note--error" role="alert">{form.shareError}</p>
			{/if}
			{#if data.shares}
				<ShareList shares={data.shares} />
			{/if}
			<p class="hint">{m.account_shares_hint()}</p>
		</section>

		<section class="section" aria-labelledby="delete-title">
			<h2 id="delete-title" class="section__label">{m.account_delete_title()}</h2>
			{#if form && 'deleteError' in form && form.deleteError}
				<p class="note note--error" role="alert">{form.deleteError}</p>
			{/if}
			{#if !deleting && !deleteChallenge}
				<div class="danger">
					<p class="hint">{m.account_delete_hint()}</p>
					<div><Button variant="outlined" size="s" onclick={() => (deleting = true)}>{m.account_delete()}</Button></div>
				</div>
			{:else}
				<form method="POST" action="?/deleteAccount" use:enhance class="password danger">
					<p class="hint">{m.account_delete_warning()}</p>
					{#if deleteChallenge}
						<p class="hint">{m.account_email_code_sent({ email: form && 'deleteSentTo' in form && form.deleteSentTo ? String(form.deleteSentTo) : '' })}</p>
						<input type="hidden" name="challenge_id" value={deleteChallenge} />
						<TextField id="delete_code" name="code" label={m.account_code()} autocomplete="one-time-code" inputmode="numeric" required />
					{:else if methods?.password}
						<TextField id="delete_password" name="password" type="password" label={m.account_password_current()} autocomplete="current-password" required />
					{/if}
					<Checkbox name="confirm" bind:checked={deleteConfirmed} label={m.account_delete_confirm()} />
					<div class="actions">
						{#if deleteChallenge || methods?.password}
							<Button type="submit" variant="filled" size="s" disabled={!deleteConfirmed}>{m.account_delete()}</Button>
						{/if}
						{#if !deleteChallenge}
							<Button type="submit" variant={methods?.password ? 'text' : 'tonal'} size="s" formaction="?/deleteCode" formnovalidate>{m.account_delete_email_code()}</Button>
						{/if}
						<Button variant="text" size="s" onclick={() => (deleting = false)}>{m.account_cancel()}</Button>
					</div>
				</form>
			{/if}
		</section>
	</div>
</div>

<style>
	.account {
		height: 100%;
		overflow-y: auto;
		padding: 24px clamp(14px, 4vw, 44px) 44px;
		box-sizing: border-box;
	}
	.account__inner {
		display: flex;
		flex-direction: column;
		gap: 22px;
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
	.section__sub {
		margin: 0 8px;
		color: var(--md-sys-color-on-surface);
		font-size: 0.9375rem;
		font-weight: 600;
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
		gap: 12px;
		min-height: 64px;
		padding: 10px 12px 10px 14px;
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
		font-size: 0.9375rem;
		font-weight: 600;
	}
	.row__supporting {
		overflow-wrap: anywhere;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.note {
		margin: 0;
		padding: 12px 14px;
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
	.password {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 14px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.password__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
		font-size: 0.9375rem;
		font-weight: 600;
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
	.email-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.picker {
		max-width: 45%;
		font: inherit;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface);
		background: var(--md-sys-color-surface-container-high);
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		padding: 8px 10px;
	}
	.danger {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	form.danger {
		outline: 1px solid var(--md-sys-color-error);
		outline-offset: -1px;
	}
</style>
