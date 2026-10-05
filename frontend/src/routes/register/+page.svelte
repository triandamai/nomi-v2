<script lang="ts">
	import { enhance } from '$app/forms';
	import AuthShell from '$lib/components/AuthShell.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import type { ActionData } from './$types';

	let { form }: { form: ActionData } = $props();
</script>

<AuthShell title="Create your account">
	<form method="POST" use:enhance class="flex flex-col gap-4">
		{#if form?.error}
			<p class="md-body-medium" role="alert" style="color: var(--md-sys-color-error)">{form.error}</p>
		{/if}
		<TextField id="email" name="email" type="email" label="Email" required />
		<TextField id="password" name="password" type="password" label="Password" required />
		<TextField id="orgName" name="orgName" type="text" label="Organization name" required />
		<Button type="submit" variant="gradient" size="m" class="w-full">Register</Button>
		<p class="md-body-medium text-center" style="color: var(--md-sys-color-on-surface-variant)">
			Already have an account?
			<a href="/login" style="color: var(--md-sys-color-primary); font-weight: 600">Log in</a>
		</p>
	</form>
</AuthShell>
