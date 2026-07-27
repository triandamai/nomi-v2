<script lang="ts">
	import { enhance } from '$app/forms';
	import type { ActionData } from './$types';

	let { form }: { form: ActionData } = $props();
</script>

<div class="flex min-h-screen items-center justify-center bg-neutral-50">
	<form
		method="POST"
		use:enhance
		class="w-full max-w-sm space-y-4 rounded-2xl border border-neutral-200 bg-white p-8 shadow-sm"
	>
		<h1 class="text-2xl font-semibold">Welcome back</h1>
		{#if form?.error}
			<p class="text-sm text-red-600">{form.error}</p>
		{/if}
		<div>
			<label for="email" class="block text-sm font-medium text-neutral-700">Email</label>
			<input
				id="email"
				name="email"
				type="email"
				required
				class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2"
			/>
		</div>
		<div>
			<label for="password" class="block text-sm font-medium text-neutral-700">Password</label>
			<input
				id="password"
				name="password"
				type="password"
				required
				class="mt-1 w-full rounded-lg border border-neutral-300 px-3 py-2"
			/>
		</div>
		<button
			type="submit"
			class="w-full rounded-lg bg-neutral-900 px-4 py-2 font-medium text-white hover:bg-neutral-800"
		>
			Log in
		</button>
		<p class="text-center text-sm text-neutral-500">
			No account? <a href="/register" class="underline">Register</a>
		</p>
	</form>
</div>
