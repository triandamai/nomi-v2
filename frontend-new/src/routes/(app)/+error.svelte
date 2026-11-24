<script lang="ts">
	import { Building2, FolderX, MessageSquareOff } from '@lucide/svelte';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import StatusPage from '$lib/errors/StatusPage.svelte';
	import { invalidateAll } from '$app/navigation';
	import { phoneView } from '$lib/phone.svelte';
	import PhoneBackBar from '$lib/ds/chrome/PhoneBackBar.svelte';
	import LargeTitle from '$lib/ds/chrome/LargeTitle.svelte';
	import PhoneLoadError from '$lib/ds/settings/PhoneLoadError.svelte';

	// A page in the app that couldn't open, inside the shell so the way on is right there: a chat
	// that isn't there (deleted, or someone else's: the backend answers both alike) says so; anything
	// else is Nomi's own 404 or error page, the crew acting it out, inside the shell. The pen draws
	// no error page; the missing ones are the design system's empty state.
	const missingChat = $derived(page.status === 404 && page.route.id === '/(app)/chat/[id]');
	// A workspace the person isn't in (or that isn't there) says so, and leads back to Workspaces.
	const missingWorkspace = $derived(page.status === 404 && page.route.id === '/(app)/settings/workspaces/[id]');
	// A project that isn't there (or not theirs) says so, and leads back to Projects.
	const missingProject = $derived(page.status === 404 && page.route.id === '/(app)/projects/[id]');

	// On a phone, a settings page that couldn't load (Flow / Mobile — You & Settings, step 23) keeps
	// its back bar to You and its title, says what happened, and offers Try again.
	const guess = () => (page.data as { phone?: boolean }).phone ?? false;
	const phone = phoneView(guess());
	const SETTINGS: Record<string, () => string> = {
		'/(app)/settings/general': m.settings_general,
		'/(app)/settings/profile': m.settings_profile,
		'/(app)/settings/plan': m.settings_plan,
		'/(app)/settings/crew': m.settings_crew,
		'/(app)/settings/models': m.settings_models,
		'/(app)/settings/memory': m.settings_memory,
		'/(app)/settings/workspaces': m.settings_workspaces,
		'/(app)/settings/workspaces/[id]': m.settings_workspaces,
		'/(app)/settings/connections': m.settings_connections,
	};
	const settingsTitle = $derived(page.status !== 404 ? SETTINGS[page.route.id ?? '']?.() : undefined);
	let retrying = $state(false);
	async function retry() {
		retrying = true;
		await invalidateAll();
		retrying = false;
	}
</script>

<svelte:head><title>{missingChat ? m.chat_missing_title() : missingWorkspace ? m.workspace_missing_title() : missingProject ? m.workspace_project_missing_title() : page.status === 404 ? m.error_404_title() : m.error_500_title()} · {m.app_name()}</title></svelte:head>

{#if phone.current && settingsTitle}
	<div class="-mx-4 -mt-6 flex flex-col">
		<PhoneBackBar label={m.nav_you()} href="/settings" />
		<LargeTitle title={settingsTitle} />
		<PhoneLoadError title={page.error?.message || m.error_title()} body={m.phone_error_body({ status: page.status })} retry={m.error_retry()} busy={retrying} onretry={retry} />
	</div>
{:else if !missingChat && !missingProject && !missingWorkspace}
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
