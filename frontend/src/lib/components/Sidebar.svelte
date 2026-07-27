<script lang="ts">
	import { enhance } from '$app/forms';
	import SessionListItem from './SessionListItem.svelte';
	import type { SessionSummary } from '$lib/types';

	let { sessions, userEmail }: { sessions: SessionSummary[]; userEmail: string } = $props();
</script>

<aside class="flex w-72 flex-col border-r border-neutral-200 bg-white">
	<div class="flex items-center gap-2 border-b border-neutral-200 px-4 py-4">
		<span class="text-lg font-semibold">Nomi</span>
	</div>

	<form method="POST" action="/?/newChat" use:enhance class="px-3 pt-3">
		<button
			type="submit"
			class="w-full rounded-lg bg-neutral-900 px-4 py-2 text-sm font-medium text-white hover:bg-neutral-800"
		>
			+ New Chat
		</button>
	</form>

	<nav class="flex-1 space-y-1 overflow-y-auto px-3 py-3">
		{#each sessions as session (session.id)}
			<SessionListItem {session} />
		{/each}
	</nav>

	<form method="POST" action="/logout" class="flex items-center justify-between border-t border-neutral-200 px-4 py-3">
		<span class="truncate text-sm text-neutral-600">{userEmail}</span>
		<button type="submit" class="text-sm text-neutral-400 hover:text-neutral-700">Log out</button>
	</form>
</aside>
