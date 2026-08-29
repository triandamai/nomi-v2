<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/stores';
	import Button from '$lib/components/m3/Button.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<div class="flex h-full flex-col items-center justify-center gap-4 text-center">
	{#if $page.url.searchParams.get('error') === 'forbidden'}
		<p class="md-body-medium" style="color: var(--md-sys-color-error)">
			That account does not have admin access.
		</p>
	{/if}
	<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Hi, {data.userEmail}!</h1>
	<p class="md-body-large" style="color: var(--md-sys-color-on-surface-variant)">How can I assist you today?</p>
	<form method="POST" action="?/newChat" use:enhance>
		<Button type="submit" variant="filled">New Chat</Button>
	</form>
</div>
