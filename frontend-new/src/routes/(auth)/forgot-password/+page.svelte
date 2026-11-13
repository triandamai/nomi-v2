<script lang="ts">
	// Forgot password, in the sign-in layout: the account's email, then the same answer whether or
	// not there's an account for it (the API never says), with the webmail shortcut as on Enter code.
	import { enhance } from '$app/forms';
	import { ArrowRight, Mail } from '@lucide/svelte';
	import AuthButton from '$lib/ds/auth/AuthButton.svelte';
	import AuthField from '$lib/ds/auth/AuthField.svelte';
	import AuthLink from '$lib/ds/auth/AuthLink.svelte';
	import MailAppButton from '$lib/ds/auth/MailAppButton.svelte';
	import ErrorCallout from '$lib/ds/conversation/ErrorCallout.svelte';
	import { alertCode, sayAlert, sayField } from '$lib/auth/say';
	import { m } from '$lib/paraglide/messages';

	let { data, form } = $props();
	let busy = $state(false);
	// The action answers either where the link went or what was wrong.
	const sent = $derived(form?.sent ? { email: form.sent, mailApp: form.mailApp ?? null } : null);
</script>

<svelte:head><title>{m.forgot_page_title()}</title></svelte:head>

{#if sent}
	<div class="flex flex-col gap-1.5">
		<h1 class="text-display font-medium text-text">{m.verify_title()}</h1>
		<p class="text-body leading-normal text-text-2" role="status">{m.forgot_sent_sub({ email: sent.email })}</p>
	</div>
	{#if sent.mailApp}<div class="flex gap-2"><MailAppButton app={sent.mailApp} /></div>{/if}
	<p class="flex flex-wrap items-center gap-x-4">
		<AuthLink label={m.auth_back_to_sign_in()} href="/sign-in" />
		<AuthLink label={m.forgot_other_email()} href="/forgot-password" quiet />
		<span class="text-caption text-text-3">{m.verify_spam()}</span>
	</p>
{:else}
	<div class="flex flex-col gap-1.5">
		<h1 class="text-display font-medium text-text">{m.forgot_title()}</h1>
		<p class="text-body leading-normal text-text-2">{m.forgot_sub()}</p>
	</div>
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
			error={sayField(form?.fields?.email)}
		/>
		<AuthButton label={m.forgot_submit()} iconEnd={ArrowRight} loading={busy} />
	</form>
	<p class="flex flex-wrap gap-x-4">
		<AuthLink label={m.auth_back_to_sign_in()} href="/sign-in" />
	</p>
{/if}
