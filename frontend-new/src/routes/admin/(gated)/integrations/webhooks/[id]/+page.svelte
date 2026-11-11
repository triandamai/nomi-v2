<script lang="ts">
	import { RotateCcw } from '@lucide/svelte';
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { can } from '$lib/admin/access';
	import type { Delivery } from '$lib/admin/api';
	import { dateTime } from '$lib/admin/format';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import AdminTable, { type TableColumn } from '$lib/ds/admin/AdminTable.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Outcome from '$lib/admin/ui/Outcome.svelte';
	import { goto } from '$app/navigation';

	// Integrations → an endpoint's deliveries (integrations: view): each event sent to it, newest
	// first, with its status, tries and what the endpoint answered; narrowed by status in the
	// address; write sends one again.
	let { data, form } = $props();

	const language = getLocale();
	const canWrite = $derived(can(data.access, 'integrations', 'write'));
	const statuses = [
		{ value: '', label: m.admin_deliveries_all() },
		{ value: 'failed', label: 'failed' },
		{ value: 'delivered', label: 'delivered' },
		{ value: 'pending', label: 'pending' },
	];
	const columns: TableColumn<Delivery>[] = [
		{ key: 'event', label: m.admin_col_event() },
		{ key: 'status', label: m.admin_col_status() },
		{ key: 'tries', label: m.admin_col_attempts(), align: 'end', phone: false },
		{ key: 'answer', label: m.admin_col_answer(), phone: false },
		{ key: 'sent', label: m.admin_col_sent(), sort: (row) => row.created_at },
		{ key: 'again', label: '', align: 'end' },
	];
</script>

<PageHeader title={m.admin_endpoint_deliveries()} meta={data.endpoint?.url} intro={m.admin_deliveries_intro()} back={{ href: '/admin/integrations', label: m.admin_deliveries_back() }} />

<Segmented label={m.admin_deliveries_status()} options={statuses} value={data.status} onchange={(status) => goto(status ? `?status=${status}` : '?', { keepFocus: true })} />
<Outcome {form} action="redeliver" done={m.admin_delivery_redelivered()} failed={m.admin_delivery_failed()} />

{#if data.deliveries.ok}
	{#if data.deliveries.data.length === 0}
		<p class="text-ui text-text-3">{m.admin_deliveries_empty()}</p>
	{:else}
		<AdminTable label={m.admin_endpoint_deliveries()} {columns} rows={data.deliveries.data} rowKey={(row) => row.id}>
			{#snippet cell(row, column)}
				{#if column.key === 'event'}
					<span class="font-mono text-text">{row.event_type}</span>
				{:else if column.key === 'status'}
					<Tag kind={row.status === 'failed' ? 'no-go' : 'neutral'} label={row.status} />
				{:else if column.key === 'tries'}
					<span class="font-mono">{row.attempts}</span>
				{:else if column.key === 'answer'}
					<span class="font-mono text-label text-text-2">{[row.last_status_code, row.last_error].filter(Boolean).join(' · ') || '—'}</span>
				{:else if column.key === 'sent'}
					<span class="font-mono text-text-2">{dateTime(row.created_at, language)}</span>
				{:else if column.key === 'again' && canWrite}
					<form method="POST" action="?/redeliver" use:enhance class="flex justify-end">
						<input type="hidden" name="id" value={row.id} />
						<Button label={m.admin_delivery_redeliver()} variant="ghost" iconStart={RotateCcw} type="submit" aria-label={m.admin_delivery_redeliver_named({ event: row.event_type })} />
					</form>
				{/if}
			{/snippet}
		</AdminTable>
	{/if}
{:else}
	<LoadFailed title={m.admin_deliveries_failed()} problem={data.deliveries.problem} />
{/if}
