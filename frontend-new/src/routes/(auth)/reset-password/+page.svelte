<script lang="ts">
	// The emailed reset link: a new password, typed twice, in the sign-in layout. A link that's spent
	// says so and offers a new one; success goes to sign in, which says the password changed.
	import { enhance } from '$app/forms';
	import { ArrowRight, LockKeyhole } from '@lucide/svelte';
	import AuthButton from '$lib/ds/auth/AuthButton.svelte';
	import AuthField from '$lib/ds/auth/AuthField.svelte';
	import AuthLink from '$lib/ds/auth/AuthLink.svelte';
	import ErrorCallout from '$lib/ds/conversation/ErrorCallout.svelte';
	import { alertCode, sayAlert, sayField } from '$lib/auth/say';
	import { m } from '$lib/paraglide/messages';

	let { data, form } = $props();
	let busy = $state(false);
	const expired = $derived(data.expired || Boolean(form?.expired));
</script>

<svelte:head><title>{m.reset_page_title()}</title></svelte:head>

{#if expired}
	<div class="flex flex-col gap-1.5">
		<h1 class="text-display font-medium text-text">{m.reset_expired_title()}</h1>
		<p class="text-body leading-normal text-text-2">{m.reset_expired_sub()}</p>
	</div>
	<AuthButton label={m.reset_new_link()} href="/forgot-password" iconEnd={ArrowRight} />
{:else}
	<div class="flex flex-col gap-1.5">
		<h1 class="text-display font-medium text-text">{m.reset_title()}</h1>
		<p class="text-body leading-normal text-text-2">{m.reset_sub()}</p>
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
		<input type="hidden" name="token" value={data.token} />
		<AuthField
			label={m.reset_password()}
			type="password"
			name="password"
			icon={LockKeyhole}
			autocomplete="new-password"
			helper={m.register_password_help()}
			error={sayField(form?.fields.password)}
		/>
		<AuthField label={m.reset_confirm()} type="password" name="confirm" icon={LockKeyhole} autocomplete="new-password" error={sayField(form?.fields.confirm)} />
		<AuthButton label={m.reset_submit()} iconEnd={ArrowRight} loading={busy} />
	</form>
{/if}

<p class="flex flex-wrap gap-x-4">
	<AuthLink label={m.auth_back_to_sign_in()} href="/sign-in" />
</p>
