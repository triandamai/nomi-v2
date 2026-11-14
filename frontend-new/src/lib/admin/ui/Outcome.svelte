<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import type { Problem } from '$lib/admin/api';
	import Notice from '$lib/ds/admin/Notice.svelte';

	// What one form action just did, under its form: saved (`done`), or failed with the backend's
	// own reason quoted. Shows only for the action it is given (a page has several forms).
	type Result = { action?: string; ok?: boolean; problem?: Problem } | null | undefined;
	type Props = { form: Result; action: string | string[]; done?: string; failed: string };
	let { form, action, done, failed }: Props = $props();

	const mine = $derived(Boolean(form?.action && (Array.isArray(action) ? action.includes(form.action) : form.action === action)));
	const reason = $derived(form?.problem ? (form.problem.status === 503 && !form.problem.message ? m.admin_unreachable() : form.problem.message) : undefined);
</script>

{#if mine && form?.ok && done}
	<Notice kind="success" text={done} />
{:else if mine && form && !form.ok}
	<Notice kind="error" text={failed} detail={reason || undefined} />
{/if}
