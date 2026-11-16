<script lang="ts">
	// Auth / Sign in or create (vDaAQ): Google first, then the email alone, "Continue with email",
	// which emails a code that signs in or sets a new address up. "Use a password instead" is the
	// email and password form (?with=password), with the ways back: the code, a new account, a
	// forgotten password.
	import { enhance } from '$app/forms';
	import { ArrowRight, KeyRound, LockKeyhole, Mail } from '@lucide/svelte';
	import AuthButton from '$lib/ds/auth/AuthButton.svelte';
	import AuthField from '$lib/ds/auth/AuthField.svelte';
	import AuthLink from '$lib/ds/auth/AuthLink.svelte';
	import AuthNote from '$lib/ds/auth/AuthNote.svelte';
	import GoogleButton from '$lib/ds/auth/GoogleButton.svelte';
	import OrDivider from '$lib/ds/auth/OrDivider.svelte';
	import ErrorCallout from '$lib/ds/conversation/ErrorCallout.svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import { withRedirect } from '$lib/auth/redirects';
	import { alertCode, sayAlert, sayField, sayGoogle } from '$lib/auth/say';
	import { m } from '$lib/paraglide/messages';

	let { data, form } = $props();
	let busy = $state(false);
	let toGoogle = $state(false);
	const method = $derived(form?.method ?? data.method);
	const password = $derived(method === 'password');
	/** The other form, keeping where the person is going and the email typed so far. */
	function otherWay(typed: string): string {
		const params = new URLSearchParams();
		if (!password) params.set('with', 'password');
		if (data.redirectTo) params.set('redirect_to', data.redirectTo);
		if (typed.trim()) params.set('email', typed.trim());
		const query = params.toString();
		return query ? `/sign-in?${query}` : '/sign-in';
	}
	const initial = () => form?.email ?? data.email;
	let email = $state(initial());
</script>

<svelte:head><title>{m.signin_page_title()}</title></svelte:head>

<div class="flex flex-col gap-1.5">
	<h1 class="text-display font-medium text-text">{password ? m.signin_title() : m.signin_create_title()}</h1>
	<p class="text-body leading-normal text-text-2">{password ? m.signin_sub() : m.signin_create_sub()}</p>
</div>

{#if data.passwordChanged}<AuthNote text={m.signin_reset_done()} announce />{/if}
{#if data.accountClosed}<AuthNote text={m.signin_account_closed()} announce />{/if}
{#if data.googleError}<ErrorCallout title={sayGoogle(data.googleError)} code="google · {data.googleError}" />{/if}

{#if data.google}
	<form method="POST" action="/auth/google" onsubmit={() => (toGoogle = true)}>
		<input type="hidden" name="redirect_to" value={data.redirectTo ?? ''} />
		<GoogleButton lastUsed={data.lastUsed === 'google'} loading={toGoogle} />
	</form>
	<OrDivider />
{/if}

<form
	method="POST"
	novalidate
	class="flex flex-col gap-5"
	use:enhance={() => {
		busy = true;
		return async ({ update }) => {
			await update({ reset: false });
			busy = false;
		};
	}}
>
	<input type="hidden" name="method" value={method} />
	{#if form?.alert}<ErrorCallout title={sayAlert(form.alert)} code={alertCode(form.alert)} />{/if}
	<AuthField
		label={m.auth_email()}
		type="email"
		name="email"
		icon={Mail}
		placeholder={m.auth_email_placeholder()}
		autocomplete="email"
		bind:value={email}
		error={sayField(form?.fields.email)}
	/>
	{#if password}
		<AuthField label={m.auth_password()} type="password" name="password" icon={LockKeyhole} autocomplete="current-password" error={sayField(form?.fields.password)} />
	{/if}
	<AuthButton label={password ? m.signin_submit() : m.signin_continue_email()} iconEnd={ArrowRight} loading={busy} />
	<p class="flex items-center gap-2 text-caption text-text-3">
		<span class="inline-flex shrink-0"><Icon icon={KeyRound} size={14} /></span>
		<span>{password ? m.signin_hint() : m.signin_code_hint()}</span>
	</p>
</form>

{#if password}
	<p class="flex flex-wrap gap-x-4">
		<AuthLink label={m.signin_use_code()} href={otherWay(email)} />
		<AuthLink label={m.signin_new()} href={withRedirect('/register', data.redirectTo)} quiet />
		<AuthLink label={m.signin_forgot()} href="/forgot-password" quiet />
	</p>
{:else}
	<p class="flex">
		<AuthLink label={m.signin_use_password()} href={otherWay(email)} />
	</p>
{/if}
