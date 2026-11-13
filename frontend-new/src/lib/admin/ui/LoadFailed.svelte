<script lang="ts">
	import { page } from '$app/state';
	import { m } from '$lib/paraglide/messages';
	import type { Problem } from '$lib/admin/api';
	import ErrorCallout from '$lib/ds/conversation/ErrorCallout.svelte';

	// What an admin page shows when its data couldn't be read: which part, the status and the
	// backend's own reason (or that it couldn't be reached), and trying again.
	let { title, problem }: { title: string; problem: Problem } = $props();

	const reason = $derived(problem.status === 503 && !problem.message ? m.admin_unreachable() : problem.message);
</script>

<ErrorCallout
	{title}
	code={problem.code}
	details={[{ key: m.admin_status(), value: String(problem.status) }, ...(reason ? [{ key: m.admin_reason(), value: reason, strong: true }] : [])]}
	actions={[{ label: m.admin_retry(), href: page.url.pathname + page.url.search }]}
/>
