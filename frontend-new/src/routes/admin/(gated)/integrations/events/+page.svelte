<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import type { ReceivedEvent } from '$lib/admin/api';
	import { dateTime, full } from '$lib/admin/format';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import AdminTable, { type TableColumn } from '$lib/ds/admin/AdminTable.svelte';
	import Pager from '$lib/ds/admin/Pager.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';

	// Integrations → events received (integrations: view): what Shipyard told Nomi, newest first,
	// with the project, environment and service each was about, a page at a time.
	let { data } = $props();

	const language = getLocale();
	/** The backend's page size (admin_shipyard.rs EVENTS_PER_PAGE); the feed doesn't say it. */
	const PER_PAGE = 20;
	const columns: TableColumn<ReceivedEvent>[] = [
		{ key: 'type', label: m.admin_col_event() },
		{ key: 'project', label: m.admin_col_project() },
		{ key: 'where', label: m.admin_col_where(), phone: false },
		{ key: 'received', label: m.admin_col_received() },
	];
</script>

<PageHeader title={m.admin_shipyard_events_link()} intro={m.admin_events_intro()} back={{ href: '/admin/integrations', label: m.admin_deliveries_back() }} />

{#if data.feed.ok}
	{@const feed = data.feed.data}
	{#if feed.events.length === 0}
		<p class="text-ui text-text-3">{m.admin_events_empty()}</p>
	{:else}
		<AdminTable label={m.admin_events_table()} {columns} rows={feed.events} rowKey={(row) => row.id}>
			{#snippet cell(row, column)}
				{#if column.key === 'type'}
					<span class="font-mono text-text">{row.type}</span>
				{:else if column.key === 'project'}
					{row.project_name ?? '—'}
				{:else if column.key === 'where'}
					<span class="font-mono text-label text-text-2">{[row.environment, row.service].filter(Boolean).join(' · ') || '—'}</span>
				{:else if column.key === 'received'}
					<span class="font-mono text-text-2">{dateTime(row.received_at, language)}</span>
				{/if}
			{/snippet}
		</AdminTable>
		<Pager
			label={m.admin_events_pages()}
			summary={m.admin_pager_summary({ from: full((feed.page - 1) * PER_PAGE + 1, language), to: full((feed.page - 1) * PER_PAGE + feed.events.length, language), total: full(feed.total, language) })}
			previousLabel={m.admin_previous()}
			nextLabel={m.admin_next()}
			previousHref={feed.page > 1 ? `?page=${feed.page - 1}` : undefined}
			nextHref={feed.page < feed.pages ? `?page=${feed.page + 1}` : undefined}
		/>
	{/if}
{:else}
	<LoadFailed title={m.admin_events_failed()} problem={data.feed.problem} />
{/if}
