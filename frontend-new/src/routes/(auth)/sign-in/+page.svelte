<script lang="ts">
	// Auth / Sign in or create (vDaAQ), following the backend as it is: Google first, then email
	// and password, which go on to the emailed code. The pen's one-step email start isn't in the API
	// yet (BACKEND_TODO.md), so the form asks for the password and "No password to remember" is out.
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
</script>

<svelte:head><title>{m.signin_page_title()}</title></svelte:head>

<div class="flex flex-col gap-1.5">
	<h1 class="text-display font-medium text-text">{m.signin_title()}</h1>
	<p class="text-body leading-normal text-text-2">{m.signin_sub()}</p>
</div>

{#if data.passwordChanged}<AuthNote text={m.signin_reset_done()} announce />{/if}
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
	{#if form?.alert}<ErrorCallout title={sayAlert(form.alert)} code={alertCode(form.alert)} />{/if}
	<AuthField
		label={m.auth_email()}
		type="email"
		name="email"
		icon={Mail}
		placeholder={m.auth_email_placeholder()}
		autocomplete="email"
		value={form?.email ?? data.email}
		error={sayField(form?.fields.email)}
	/>
	<AuthField label={m.auth_password()} type="password" name="password" icon={LockKeyhole} autocomplete="current-password" error={sayField(form?.fields.password)} />
	<AuthButton label={m.signin_submit()} iconEnd={ArrowRight} loading={busy} />
	<p class="flex items-center gap-2 text-caption text-text-3">
		<span class="inline-flex shrink-0"><Icon icon={KeyRound} size={14} /></span>
		<span>{m.signin_hint()}</span>
	</p>
</form>

<p class="flex flex-wrap gap-x-4">
	<AuthLink label={m.signin_new()} href={withRedirect('/register', data.redirectTo)} />
	<AuthLink label={m.signin_forgot()} href="/forgot-password" quiet />
</p>
