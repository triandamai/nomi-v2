<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { page } from '$app/state';
	import { enhance } from '$app/forms';
	import { FormPending } from '$lib/formPending.svelte';
	import AuthShell from '$lib/components/AuthShell.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import Snackbar from '$lib/components/m3/Snackbar.svelte';
	import type { ActionData } from './$types';

	let { form }: { form: ActionData } = $props();

	let toastOpen = $state(false);
	let toast = $state('');
	const sending = new FormPending({ reset: false });
	// "Use a different email" goes back to the form without a reload.
	let again = $state(false);
	const sent = $derived(!again && form && 'sent' in form ? (form.sent as string) : null);
	const emailError = $derived(form && 'field' in form && form.field === 'email' ? form.error : undefined);
	const formError = $derived(form && 'field' in form && !form.field ? form.error : undefined);

	$effect(() => {
		if (!form) return;
		again = false;
		toast = ('error' in form && form.error) || m.forgot_sent_toast();
		toastOpen = true;
	});
</script>

<AuthShell
	title={sent ? m.forgot_sent_title() : m.forgot_title()}
	subtitle={sent ? m.forgot_sent_body({ email: sent }) : m.forgot_subtitle()}
	switchPrompt={m.register_have_account()}
	switchLabel={m.login_submit()}
	switchHref={'/login' + page.url.search}
>
	{#if sent}
		<div class="flex flex-col gap-3">
			<Button variant="gradient" size="m" class="w-full" href={'/login' + page.url.search}>{m.verify_back_to_sign_in()}</Button>
			<Button variant="text" size="m" class="w-full" onclick={() => (again = true)}>{m.forgot_again()}</Button>
		</div>
	{:else}
		<form
			method="POST"
			class="flex flex-col gap-5"
			use:enhance={sending.submit}
		>
			{#if formError}
				<p class="md-body-medium" role="alert" style="color: var(--md-sys-color-error)">{formError}</p>
			{/if}
			<TextField
				id="email"
				name="email"
				type="email"
				label={m.auth_email()}
				autocomplete="email"
				value={page.url.searchParams.get('email') ?? ''}
				error={Boolean(emailError)}
				supportingText={emailError}
				required
			/>
			<Button type="submit" variant="gradient" size="m" class="mt-2 w-full" loading={sending.on}>{m.forgot_submit()}</Button>
		</form>
	{/if}
</AuthShell>
<Snackbar bind:open={toastOpen} message={toast} />
