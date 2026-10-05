<script lang="ts">
	import { enhance } from '$app/forms';
	import AuthShell from '$lib/components/AuthShell.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import type { ActionData } from './$types';

	let { form }: { form: ActionData } = $props();
</script>

<AuthShell
	title="Create your account"
	subtitle="Takes a minute. Nomi and the crew are ready when you are."
	switchPrompt="Have an account?"
	switchLabel="Log in"
	switchHref="/login"
>
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
