<script lang="ts">
	import { enhance } from '$app/forms';
	import AuthShell from '$lib/components/AuthShell.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import type { ActionData } from './$types';

	let { form }: { form: ActionData } = $props();
</script>

<AuthShell
	title="Welcome back"
	subtitle="Log in and pick up where your crew left off."
	switchPrompt="New here?"
	switchLabel="Create account"
	switchHref="/register"
>
	<form method="POST" use:enhance class="flex flex-col gap-5">
		{#if form?.error}
			<p class="md-body-medium" role="alert" style="color: var(--md-sys-color-error)">{form.error}</p>
		{/if}
		<TextField id="email" name="email" type="email" label="Email" autocomplete="email" required />
		<TextField id="password" name="password" type="password" label="Password" autocomplete="current-password" required />
		<Button type="submit" variant="gradient" size="m" class="mt-2 w-full">Log in</Button>
	</form>
</AuthShell>
