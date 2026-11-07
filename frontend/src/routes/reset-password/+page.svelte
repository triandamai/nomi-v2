<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import { FormPending } from '$lib/formPending.svelte';
	import AuthShell from '$lib/components/AuthShell.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import Snackbar from '$lib/components/m3/Snackbar.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let toastOpen = $state(false);
	const sending = new FormPending({ reset: false });
	const expired = $derived(data.expired || Boolean(form && 'expired' in form && form.expired));
	const errorFor = (field: string) => (form?.field === field ? form.error : undefined);
	const formError = $derived(form?.error && !form.field ? form.error : undefined);

	$effect(() => {
		if (form?.error) toastOpen = true;
	});
</script>

<AuthShell
	title={m.reset_title()}
	subtitle={expired ? m.reset_expired() : m.reset_subtitle()}
	switchPrompt={m.register_have_account()}
	switchLabel={m.login_submit()}
	switchHref="/login"
>
	{#if expired}
		<Button variant="gradient" size="m" class="w-full" href="/forgot-password">{m.reset_request_new()}</Button>
	{:else}
		<form
			method="POST"
			class="flex flex-col gap-5"
			use:enhance={sending.submit}
		>
			<input type="hidden" name="token" value={data.token} />
			{#if formError}
				<p class="md-body-medium" role="alert" style="color: var(--md-sys-color-error)">{formError}</p>
			{/if}
			<TextField
				id="password"
				name="password"
				type="password"
				label={m.auth_new_password()}
				autocomplete="new-password"
				error={Boolean(errorFor('password'))}
				supportingText={errorFor('password')}
				required
			/>
			<TextField
				id="confirmPassword"
				name="confirmPassword"
				type="password"
				label={m.auth_confirm_password()}
				autocomplete="new-password"
				error={Boolean(errorFor('confirm'))}
				supportingText={errorFor('confirm')}
				required
			/>
			<Button type="submit" variant="gradient" size="m" class="mt-2 w-full" loading={sending.on}>{m.reset_submit()}</Button>
		</form>
	{/if}
</AuthShell>
<Snackbar bind:open={toastOpen} message={form?.error ?? ''} />
