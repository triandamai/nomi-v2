<script lang="ts">
	import SessionListItem from '$lib/components/SessionListItem.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();
</script>

<div class="h-full overflow-y-auto p-8">
	<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Chats</h1>

	<div class="mt-6 space-y-1">
		{#each data.sessions as session (session.id)}
			<SessionListItem {session} />
		{/each}
		{#if data.sessions.length === 0}
			<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">No chats yet.</p>
		{/if}
	</div>
</div>
