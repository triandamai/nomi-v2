<script lang="ts">
	import { enhance } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<div class="flex h-full flex-col items-center justify-center gap-4 text-center">
	<h1 class="md-headline-small" style="color: var(--md-sys-color-on-surface)">Hi, {data.userEmail}!</h1>
	<p class="md-body-large" style="color: var(--md-sys-color-on-surface-variant)">How can I assist you today?</p>
	<form method="POST" action="?/newChat" use:enhance>
		<Button type="submit" variant="filled">New Chat</Button>
	</form>
</div>
