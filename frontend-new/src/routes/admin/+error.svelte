<script lang="ts">
	import { Lock, SearchX } from '@lucide/svelte';
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import { NO_ADMIN_ACCESS } from '$lib/admin/access';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import ErrorCallout from '$lib/ds/conversation/ErrorCallout.svelte';

	// The console's own states for what can't be shown: no admin area at all (the gate's 403), an
	// area the backend refused (a 403 from a page's load: access changed, or the token is older than
	// a grant), a page that doesn't exist, and anything else with the backend's own reason quoted.
	const status = $derived(page.status);
	const message = $derived(page.error?.message ?? '');
</script>

<svelte:head><title>{m.admin_title()} · {m.app_name()}</title></svelte:head>

<div class="flex w-full max-w-140 flex-col gap-3 pt-12">
	{#if status === 403 && message === NO_ADMIN_ACCESS}
		<EmptyState icon={Lock} title={m.admin_forbidden_title()} body={m.admin_forbidden_body()} action={{ label: m.admin_back_to_app(), href: '/' }} />
	{:else if status === 403}
		<EmptyState icon={Lock} title={m.admin_denied_title()} body={m.admin_denied_body()} action={{ label: m.admin_denied_home(), href: '/admin' }} />
		{#if message}<p class="font-mono text-label text-text-3">{message}</p>{/if}
	{:else if status === 404}
		<EmptyState icon={SearchX} title={m.admin_not_found_title()} body={m.admin_not_found_body()} action={{ label: m.admin_denied_home(), href: '/admin' }} />
	{:else}
		<ErrorCallout
			title={m.admin_error_title()}
			details={[
				{ key: m.admin_status(), value: String(status) },
				...(message ? [{ key: m.admin_reason(), value: message, strong: true }] : []),
			]}
			actions={[{ label: m.admin_retry(), href: page.url.pathname + page.url.search }]}
		/>
	{/if}
</div>
