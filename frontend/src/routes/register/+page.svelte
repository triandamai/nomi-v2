<script lang="ts">
	import { enhance } from '$app/forms';
	import AuthShell from '$lib/components/AuthShell.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import GoogleButton from '$lib/components/GoogleButton.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();
</script>

<AuthShell
	title="Create your account"
	subtitle="Takes a minute. Nomi and the crew are ready when you are."
	switchPrompt="Have an account?"
	switchLabel="Log in"
	switchHref="/login"
>
	{#if data.googleError}
		<p class="md-body-medium mb-4" role="alert" style="color: var(--md-sys-color-error)">{data.googleError}</p>
	{/if}
	{#if data.google}
		<GoogleButton label="Sign up with Google" />
		<div class="or" aria-hidden="true"><span>or</span></div>
	{/if}
	<form method="POST" use:enhance class="flex flex-col gap-5">
		{#if form?.error}
			<p class="md-body-medium" role="alert" style="color: var(--md-sys-color-error)">{form.error}</p>
		{/if}
		<TextField id="email" name="email" type="email" label="Email" autocomplete="email" required />
		<TextField id="password" name="password" type="password" label="Password" autocomplete="new-password" required />
		<TextField id="orgName" name="orgName" type="text" label="Organization name" required />
		<Button type="submit" variant="gradient" size="m" class="mt-2 w-full">Register</Button>
	</form>
</AuthShell>

<style>
	.or {
		display: flex;
		align-items: center;
		gap: 12px;
		margin: 20px 0;
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}
	.or::before,
	.or::after {
		content: '';
		flex: 1;
		height: 1px;
		background: var(--md-sys-color-outline-variant);
	}
</style>
