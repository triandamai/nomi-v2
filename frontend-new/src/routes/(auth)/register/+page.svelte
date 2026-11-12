<script lang="ts">
	// Create an account, in the Sign in or create layout (vDaAQ): Google first, then email, password
	// and a name for the workspace, which go on to the emailed code. Arriving from an invite, the
	// invite's code takes the workspace's place: the account joins that workspace.
	import { enhance } from '$app/forms';
	import { ArrowRight, Building2, KeyRound, LockKeyhole, Mail, Ticket } from '@lucide/svelte';
	import AuthButton from '$lib/ds/auth/AuthButton.svelte';
	import AuthField from '$lib/ds/auth/AuthField.svelte';
	import AuthLink from '$lib/ds/auth/AuthLink.svelte';
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

<svelte:head><title>{m.register_page_title()}</title></svelte:head>

<div class="flex flex-col gap-1.5">
	<h1 class="text-display font-medium text-text">{m.register_title()}</h1>
	<p class="text-body leading-normal text-text-2">{data.invite ? m.register_sub_invite() : m.register_sub()}</p>
</div>

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
	{#if form?.alert}
		<ErrorCallout
			title={sayAlert(form.alert)}
			code={alertCode(form.alert)}
			actions={form.alert.kind === 'register_email_failed' ? [{ label: m.signin_submit(), href: withRedirect('/sign-in', data.redirectTo) }] : []}
		/>
	{/if}
	<AuthField
		label={m.auth_email()}
		type="email"
		name="email"
		icon={Mail}
		placeholder={m.auth_email_placeholder()}
		autocomplete="email"
		value={form?.values.email ?? data.email}
		error={sayField(form?.fields.email)}
	/>
	<AuthField
		label={m.auth_password()}
		type="password"
		name="password"
		icon={LockKeyhole}
		autocomplete="new-password"
		helper={m.register_password_help()}
		error={sayField(form?.fields.password)}
	/>
	{#if data.invite}
		<AuthField label={m.register_invite()} icon={Ticket} value={data.invite} readonly mono helper={m.register_invite_help()} error={sayField(form?.fields.invite)} />
	{:else}
		<AuthField
			label={m.register_workspace()}
			name="workspace"
			icon={Building2}
			placeholder={m.register_workspace_placeholder()}
			autocomplete="organization"
			value={form?.values.workspace ?? ''}
			helper={m.register_workspace_help()}
			error={sayField(form?.fields.workspace)}
		/>
	{/if}
	<AuthButton label={m.register_submit()} iconEnd={ArrowRight} loading={busy} />
	<p class="flex items-center gap-2 text-caption text-text-3">
		<span class="inline-flex shrink-0"><Icon icon={KeyRound} size={14} /></span>
		<span>{m.signin_hint()}</span>
	</p>
</form>

<p class="flex flex-wrap gap-x-4">
	<AuthLink label={m.register_have_account()} href={withRedirect('/sign-in', data.redirectTo)} />
</p>
