<script lang="ts">
	// The one screen after a new account's sign-in code (Build → First run): their name, and one
	// line on what Nomi does. No crew tour: the composer comes next.
	import { deserialize } from '$app/forms';
	import { goto } from '$app/navigation';
	import FirstRun from '$lib/components/ds/build/FirstRun.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
	let error = $state<string | null>(null);

	async function save(name: string) {
		const body = new FormData();
		body.set('name', name);
		const response = await fetch('?/default', { method: 'POST', body, headers: { 'x-sveltekit-action': 'true' } });
		const result = deserialize(await response.text());
		if (result.type === 'redirect') await goto(result.location, { invalidateAll: true });
		else if (result.type === 'failure') error = String(result.data?.error ?? '');
	}
</script>

<div class="first-run">
	<FirstRun name={data.name} oncontinue={save} />
	{#if error}<p class="first-run__error" role="alert">{error}</p>{/if}
</div>

<style>
	.first-run {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		padding: 48px 16px;
	}
	.first-run__error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
</style>
