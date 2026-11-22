<script lang="ts">
	import { Building2, FolderX, MessageSquareOff } from '@lucide/svelte';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import StatusPage from '$lib/errors/StatusPage.svelte';

	// A page in the app that couldn't open, inside the shell so the way on is right there: a chat
	// that isn't there (deleted, or someone else's: the backend answers both alike) says so; anything
	// else is Nomi's own 404 or error page, the crew acting it out, inside the shell. The pen draws
	// no error page; the missing ones are the design system's empty state.
	const missingChat = $derived(page.status === 404 && page.route.id === '/(app)/chat/[id]');
	// A workspace the person isn't in (or that isn't there) says so, and leads back to Workspaces.
	const missingWorkspace = $derived(page.status === 404 && page.route.id === '/(app)/settings/workspaces/[id]');
	// A project that isn't there (or not theirs) says so, and leads back to Projects.
	const missingProject = $derived(page.status === 404 && page.route.id === '/(app)/projects/[id]');
</script>

<svelte:head><title>{missingChat ? m.chat_missing_title() : missingWorkspace ? m.workspace_missing_title() : missingProject ? m.workspace_project_missing_title() : page.status === 404 ? m.error_404_title() : m.error_500_title()} · {m.app_name()}</title></svelte:head>

{#if !missingChat && !missingProject && !missingWorkspace}
	<StatusPage status={page.status} message={page.error?.message} />
{:else}
	<div class="flex min-h-full flex-col items-center justify-center">
		<div class="w-full max-w-120">
			{#if missingChat}
				<EmptyState icon={MessageSquareOff} title={m.chat_missing_title()} body={m.chat_missing_body()} action={{ label: m.nav_home(), href: '/' }} />
			{:else if missingProject}
				<EmptyState icon={FolderX} title={m.workspace_project_missing_title()} body={m.workspace_project_missing()} action={{ label: m.nav_projects(), href: '/projects' }} />
			{:else if missingWorkspace}
				<EmptyState icon={Building2} title={m.workspace_missing_title()} body={m.workspace_not_found()} action={{ label: m.settings_workspaces(), href: '/settings/workspaces' }} />
			{/if}
		</div>
	</div>
{/if}
