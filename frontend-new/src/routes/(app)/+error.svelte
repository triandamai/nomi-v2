<script lang="ts">
	import { Building2, CircleAlert, MessageSquareOff } from '@lucide/svelte';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';

	// A page in the app that couldn't open, inside the shell so the way on is right there: a chat
	// that isn't there (deleted, or someone else's: the backend answers both alike) says so; anything
	// else says Nomi couldn't open it. Both lead Home. The pen draws no error page; this is the
	// design system's empty state.
	const missingChat = $derived(page.status === 404 && page.route.id === '/(app)/chat/[id]');
	// A workspace the person isn't in (or that isn't there) says so, and leads back to Workspaces.
	const missingWorkspace = $derived(page.status === 404 && page.route.id === '/(app)/settings/workspaces/[id]');
</script>

<svelte:head><title>{missingChat ? m.chat_missing_title() : missingWorkspace ? m.workspace_missing_title() : m.error_title()} · {m.app_name()}</title></svelte:head>

<div class="flex min-h-full flex-col items-center justify-center">
	<div class="w-full max-w-120">
		{#if missingChat}
			<EmptyState icon={MessageSquareOff} title={m.chat_missing_title()} body={m.chat_missing_body()} action={{ label: m.nav_home(), href: '/' }} />
		{:else if missingWorkspace}
			<EmptyState icon={Building2} title={m.workspace_missing_title()} body={m.workspace_not_found()} action={{ label: m.settings_workspaces(), href: '/settings/workspaces' }} />
		{:else}
			<EmptyState icon={CircleAlert} title={m.error_title()} body={m.error_body()} action={{ label: m.nav_home(), href: '/' }} />
		{/if}
	</div>
</div>
