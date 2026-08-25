<script lang="ts">
	import { enhance } from '$app/forms';
	import SessionListItem from './SessionListItem.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import type { SessionSummary } from '$lib/types';

	let { sessions, userEmail }: { sessions: SessionSummary[]; userEmail: string } = $props();
</script>

<aside
	class="flex w-72 flex-col"
	style="background: var(--md-sys-color-surface-container); border-right: 1px solid var(--md-sys-color-outline-variant)"
>
	<div class="flex items-center gap-2 px-4 py-4" style="border-bottom: 1px solid var(--md-sys-color-outline-variant)">
		<span class="md-title-large" style="color: var(--md-sys-color-primary)">Nomi</span>
	</div>

	<form method="POST" action="/?/newChat" use:enhance class="px-3 pt-3">
		<Button type="submit" variant="filled" class="w-full">+ New Chat</Button>
	</form>

	<nav class="flex-1 space-y-1 overflow-y-auto px-3 py-3">
		{#each sessions as session (session.id)}
			<SessionListItem {session} />
		{/each}
	</nav>

	<form
		method="POST"
		action="/logout"
		class="flex items-center justify-between px-4 py-3"
		style="border-top: 1px solid var(--md-sys-color-outline-variant)"
	>
		<span class="md-body-medium truncate" style="color: var(--md-sys-color-on-surface-variant)">{userEmail}</span>
		<button type="submit" class="md-label-large" style="color: var(--md-sys-color-outline); background: none; border: none; cursor: pointer">
			Log out
		</button>
	</form>
</aside>
