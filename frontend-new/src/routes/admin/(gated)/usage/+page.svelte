<script lang="ts">
	import { ChartColumn, ChevronLeft, ChevronRight } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import type { AdminUsage } from '$lib/admin/api';
	import { compact, full, usd } from '$lib/admin/format';
	import { monthLinks, quiet, usageChart } from '$lib/admin/usage';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import StatTile from '$lib/ds/admin/StatTile.svelte';
	import UsageChart from '$lib/ds/admin/UsageChart.svelte';
	import AdminTable, { type TableColumn } from '$lib/ds/admin/AdminTable.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';

	// Usage (dashboard: view): one month's tokens and spend (UTC), chat tokens per day as a chart or
	// a table, then per chat model, per embedding model and the sandbox minutes per person. Spend is
	// in US dollars at the models' prices; own-key tokens are counted, never billed.
	let { data } = $props();

	const language = getLocale();
	let view = $state('chart');

	type ModelRow = AdminUsage['models'][number];
	type EmbeddingRow = AdminUsage['embedding_models'][number];
	type SandboxRow = AdminUsage['sandbox_minutes'][number];
	type DayRow = ReturnType<typeof usageChart>['days'][number];

	const modelColumns: TableColumn<ModelRow>[] = [
		{ key: 'model', label: m.admin_col_model(), sort: (row) => row.label },
		{ key: 'tokens', label: m.admin_col_tokens(), align: 'end', sort: (row) => row.tokens },
		{ key: 'in', label: m.admin_col_in(), align: 'end', sort: (row) => row.input_tokens, phone: false },
		{ key: 'out', label: m.admin_col_out(), align: 'end', sort: (row) => row.output_tokens, phone: false },
		{ key: 'calls', label: m.admin_col_requests(), align: 'end', sort: (row) => row.calls, phone: false },
		{ key: 'people', label: m.admin_col_people(), align: 'end', sort: (row) => row.people, phone: false },
		{ key: 'spend', label: m.admin_col_spend(), align: 'end', sort: (row) => row.spend_usd },
	];
	const embeddingColumns: TableColumn<EmbeddingRow>[] = [
		{ key: 'model', label: m.admin_col_model() },
		{ key: 'tokens', label: m.admin_col_tokens(), align: 'end', sort: (row) => row.tokens },
		{ key: 'calls', label: m.admin_col_requests(), align: 'end', sort: (row) => row.calls, phone: false },
		{ key: 'people', label: m.admin_col_people(), align: 'end', sort: (row) => row.people },
	];
	const sandboxColumns: TableColumn<SandboxRow>[] = [
		{ key: 'person', label: m.admin_col_person() },
		{ key: 'minutes', label: m.admin_col_minutes(), align: 'end', sort: (row) => row.minutes },
	];
	const dayColumns: TableColumn<DayRow>[] = [
		{ key: 'day', label: m.admin_col_day() },
		{ key: 'chat', label: m.admin_col_chat_tokens(), align: 'end' },
		{ key: 'embedding', label: m.admin_col_embedding_tokens(), align: 'end' },
	];
</script>

{#if data.usage.ok}
	{@const usage = data.usage.data}
	{@const months = monthLinks(usage, language)}
	{@const chart = usageChart(usage, language)}
	<PageHeader title={m.admin_page_usage()} intro={m.admin_usage_intro()}>
		{#snippet actions()}
			<nav aria-label={m.admin_usage_months()} class="flex items-center gap-2">
				<Button label={m.admin_usage_previous()} variant="ghost" iconStart={ChevronLeft} href={months.previous} />
				<span class="min-w-32 text-center text-ui font-medium text-text" aria-current="page">{months.label}</span>
				{#if months.next}
					<Button label={m.admin_usage_next()} variant="ghost" iconEnd={ChevronRight} href={months.next} />
				{:else}
					<Button label={m.admin_usage_next()} variant="ghost" iconEnd={ChevronRight} disabled />
				{/if}
			</nav>
		{/snippet}
	</PageHeader>

	{#if quiet(usage)}
		<EmptyState icon={ChartColumn} title={m.admin_usage_quiet_title()} body={m.admin_usage_quiet_body({ month: months.label })} />
	{:else}
		<div class="grid grid-cols-2 gap-3 lg:grid-cols-4">
			<StatTile
				label={m.admin_usage_chat_tokens()}
				value={compact(usage.chat.tokens, language)}
				exact={full(usage.chat.tokens, language)}
				note={usage.chat.cached_input_tokens > 0
					? `${m.admin_usage_in_out({ input: compact(usage.chat.input_tokens, language), output: compact(usage.chat.output_tokens, language) })} · ${m.admin_usage_cached({ cached: compact(usage.chat.cached_input_tokens, language) })}`
					: m.admin_usage_in_out({ input: compact(usage.chat.input_tokens, language), output: compact(usage.chat.output_tokens, language) })}
			/>
			<StatTile
				label={m.admin_usage_spend()}
				value={usd(usage.chat.spend_usd, language)}
				note={usage.chat.own_key_tokens > 0 ? m.admin_usage_own_keys({ tokens: compact(usage.chat.own_key_tokens, language) }) : m.admin_usage_spend_note()}
			/>
			<StatTile label={m.admin_usage_embedding_tokens()} value={compact(usage.embedding.tokens, language)} exact={full(usage.embedding.tokens, language)} note={m.admin_usage_requests_note({ count: full(usage.embedding.calls, language) })} />
			<StatTile label={m.admin_usage_requests()} value={full(usage.chat.calls, language)} note={m.admin_usage_people_note({ count: full(usage.chat.people, language) })} />
		</div>

		<section aria-label={m.admin_usage_per_day()} class="flex flex-col gap-3">
			<div class="flex flex-wrap items-center justify-between gap-3">
				<SectionLabel label={m.admin_usage_per_day()} />
				<Segmented
					label={m.admin_usage_view()}
					bind:value={view}
					options={[
						{ value: 'chart', label: m.admin_usage_view_chart() },
						{ value: 'table', label: m.admin_usage_view_table() },
					]}
				/>
			</div>
			{#if view === 'chart'}
				<div class="rounded-md p-4 outline -outline-offset-1 outline-border">
					<UsageChart title={m.admin_usage_per_day()} bars={chart.bars} max={chart.max} axis={chart.axis} />
				</div>
			{:else}
				<AdminTable label={m.admin_usage_days_table()} columns={dayColumns} rows={chart.days} rowKey={(row) => row.date}>
					{#snippet cell(row, column)}
						{#if column.key === 'day'}{row.label}{:else if column.key === 'chat'}<span class="font-mono text-caption">{row.chat}</span>{:else}<span class="font-mono text-caption">{row.embedding}</span>{/if}
					{/snippet}
				</AdminTable>
			{/if}
		</section>

		<section aria-label={m.admin_usage_models()} class="flex flex-col gap-3">
			<div class="flex flex-col gap-1">
				<SectionLabel label={m.admin_usage_models()} count={usage.models.length} />
				<p class="text-caption text-text-3">{m.admin_usage_models_note()}</p>
			</div>
			{#if usage.models.length === 0}
				<p class="text-ui text-text-2">{m.admin_usage_models_empty({ month: months.label })}</p>
			{:else}
				<AdminTable label={m.admin_usage_models()} columns={modelColumns} rows={usage.models} rowKey={(row) => `${row.admin_model_id}-${row.provider}-${row.model_id}-${row.source}`}>
					{#snippet cell(row, column)}
						{#if column.key === 'model'}
							<span class="flex flex-col gap-0.5">
								<span class="inline-flex flex-wrap items-center gap-1.5">
									{row.label}
									{#if row.source === 'own_key'}<Tag kind="neutral" label={m.admin_usage_own_key()} />{/if}
									{#if row.deleted}<Tag kind="neutral" label={m.admin_usage_deleted()} />{/if}
								</span>
								<span class="font-mono text-label text-text-3">{row.provider} · {row.model_id || '—'}</span>
							</span>
						{:else if column.key === 'tokens'}
							<span class="font-mono text-caption" title={full(row.tokens, language)}>{compact(row.tokens, language)}</span>
						{:else if column.key === 'in'}
							<span class="flex flex-col items-end gap-0.5 font-mono text-caption">
								{compact(row.input_tokens, language)}
								{#if row.cached_input_tokens > 0}<span class="text-label text-text-3">{m.admin_usage_cached_short({ count: compact(row.cached_input_tokens, language) })}</span>{/if}
							</span>
						{:else if column.key === 'out'}
							<span class="font-mono text-caption">{compact(row.output_tokens, language)}</span>
						{:else if column.key === 'calls'}
							<span class="font-mono text-caption">{full(row.calls, language)}</span>
						{:else if column.key === 'people'}
							<span class="font-mono text-caption">{full(row.people, language)}</span>
						{:else}
							<span class="font-mono text-caption">{row.source === 'own_key' ? m.admin_usage_not_billed() : usd(row.spend_usd, language)}</span>
						{/if}
					{/snippet}
				</AdminTable>
			{/if}
		</section>

		<section aria-label={m.admin_usage_embedding_models()} class="flex flex-col gap-3">
			<div class="flex flex-col gap-1">
				<SectionLabel label={m.admin_usage_embedding_models()} count={usage.embedding_models.length} />
				<p class="text-caption text-text-3">{m.admin_usage_embedding_note()}</p>
			</div>
			{#if usage.embedding_models.length === 0}
				<p class="text-ui text-text-2">{m.admin_usage_embedding_empty({ month: months.label })}</p>
			{:else}
				<AdminTable label={m.admin_usage_embedding_models()} columns={embeddingColumns} rows={usage.embedding_models} rowKey={(row) => `${row.provider}-${row.model_id}`}>
					{#snippet cell(row, column)}
						{#if column.key === 'model'}
							<span class="flex flex-col gap-0.5">
								<span class="inline-flex items-center gap-1.5 font-mono text-caption">{row.model_id || row.provider}{#if row.current}<Tag kind="go" label={m.admin_usage_in_use()} />{/if}</span>
								<span class="font-mono text-label text-text-3">{row.provider}</span>
							</span>
						{:else if column.key === 'tokens'}
							<span class="flex flex-col items-end gap-0.5 font-mono text-caption">
								{compact(row.tokens, language)}
								{#if row.estimated_calls > 0}<span class="text-label text-text-3">{m.admin_usage_estimates()}</span>{/if}
							</span>
						{:else if column.key === 'calls'}
							<span class="font-mono text-caption">{full(row.calls, language)}</span>
						{:else}
							<span class="font-mono text-caption">{full(row.people, language)}</span>
						{/if}
					{/snippet}
				</AdminTable>
				{#if usage.embedding.estimated_calls > 0}<p class="text-caption text-text-3">{m.admin_usage_estimates_note()}</p>{/if}
			{/if}
		</section>

		<section aria-label={m.admin_usage_sandbox()} class="flex flex-col gap-3">
			<div class="flex flex-col gap-1">
				<SectionLabel label={m.admin_usage_sandbox()} count={usage.sandbox_minutes.length} />
				<p class="text-caption text-text-3">{m.admin_usage_sandbox_note()}</p>
			</div>
			{#if usage.sandbox_minutes.length === 0}
				<p class="text-ui text-text-2">{m.admin_usage_sandbox_empty({ month: months.label })}</p>
			{:else}
				<AdminTable label={m.admin_usage_sandbox()} columns={sandboxColumns} rows={usage.sandbox_minutes} rowKey={(row) => row.user_id}>
					{#snippet cell(row, column)}
						{#if column.key === 'person'}
							<a href="/admin/users/{row.user_id}" class="rounded-xs outline-offset-2 hover:underline">{row.email ?? m.admin_usage_no_email()}</a>
						{:else}
							<span class="font-mono text-caption">{full(row.minutes, language)}</span>
						{/if}
					{/snippet}
				</AdminTable>
			{/if}
		</section>
	{/if}
{:else}
	<PageHeader title={m.admin_page_usage()} intro={m.admin_usage_intro()} />
	<LoadFailed title={m.admin_usage_failed()} problem={data.usage.problem} />
{/if}
