<script lang="ts">
	import { enhance } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import type { ActionData } from './$types';

	let { form }: { form: ActionData } = $props();
</script>

<div
	class="flex min-h-screen items-center justify-center"
	style="background: var(--md-sys-color-surface-container-lowest)"
>
	<form
		method="POST"
		use:enhance
		class="w-full max-w-sm space-y-4 p-8"
		style="background: var(--md-sys-color-surface-container-low); border-radius: var(--md-sys-shape-corner-large); box-shadow: var(--md-sys-elevation-shadow-level2)"
	>
		<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Welcome back</h1>
		{#if form?.error}
			<p class="md-body-medium" style="color: var(--md-sys-color-error)">{form.error}</p>
		{/if}
		<TextField id="email" name="email" type="email" label="Email" required />
		<TextField id="password" name="password" type="password" label="Password" required />
		<Button type="submit" variant="filled" class="w-full">Log in</Button>
		<p class="md-body-medium text-center" style="color: var(--md-sys-color-on-surface-variant)">
			No account?
			<a href="/register" style="color: var(--md-sys-color-primary); text-decoration: underline">Register</a>
		</p>
	</form>
</div>
