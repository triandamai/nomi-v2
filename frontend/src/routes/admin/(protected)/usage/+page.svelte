<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	// Admin → Usage: everyone's tokens for a month (UTC). Chat models are listed by the model
	// record they ran on (renaming a model keeps its history together); embeddings get their own
	// table, since they don't count toward anyone's allowance.
	import { getLocale } from '$lib/paraglide/runtime';
	import DataTable from '$lib/components/m3/DataTable.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import IconChevronLeft from '$lib/components/icons/IconChevronLeft.svelte';
	import IconChevronRight from '$lib/components/icons/IconChevronRight.svelte';
	import Money from '$lib/components/Money.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { monthLabel, niceScale, shiftMonth } from '$lib/money';
	import { formatTokens, formatTokensFull, formatUsd } from '$lib/usage';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const usage = $derived(data.usage);
	const isCurrent = $derived(usage ? usage.month === usage.current_month : true);
	let showDayTable = $state(false);
	let hoverDay = $state<number | null>(null);

	const days = $derived.by(() => {
		if (!usage) return [];
		const [year, monthNumber] = usage.month.split('-').map(Number);
		const count = new Date(Date.UTC(year, monthNumber, 0)).getUTCDate();
		const byDate = new Map(usage.by_day.map((d) => [d.date, d]));
		return Array.from({ length: count }, (_, i) => {
			const date = `${usage.month}-${String(i + 1).padStart(2, '0')}`;
			const found = byDate.get(date);
			return { date, day: i + 1, chat: found?.chat_tokens ?? 0, embedding: found?.embedding_tokens ?? 0 };
		});
	});
	const dayScale = $derived(niceScale(Math.max(0, ...days.map((d) => d.chat))));
	const modelMax = $derived(Math.max(1, ...(usage?.models.map((row) => row.tokens) ?? [1])));
	const hovered = $derived(hoverDay !== null ? days[hoverDay - 1] : null);

	function dayLabel(date: string): string {
		return new Intl.DateTimeFormat(getLocale(), { weekday: 'short', day: 'numeric', month: 'short', timeZone: 'UTC' }).format(new Date(`${date}T00:00:00Z`));
	}

	const CHART_W = 640;
	const CHART_H = 220;
	const AXIS_W = 48;
	const BASE_Y = CHART_H - 26;
	const TOP_Y = 10;
	const plotW = CHART_W - AXIS_W;
	const slot = $derived(days.length ? plotW / days.length : plotW);
	const barW = $derived(Math.min(24, Math.max(2, slot - 2)));
	function y(value: number): number {
		return BASE_Y - (value / dayScale.max) * (BASE_Y - TOP_Y);
	}
	function barPath(x: number, w: number, top: number): string {
		const r = Math.min(4, w / 2, BASE_Y - top);
		return `M${x} ${BASE_Y} V${top + r} Q${x} ${top} ${x + r} ${top} H${x + w - r} Q${x + w} ${top} ${x + w} ${top + r} V${BASE_Y} Z`;
	}

	const modelColumns = [
		{ key: 'model', label: m.admin_usage_col_model() },
		{ key: 'tokens', label: m.admin_usage_col_tokens() },
		{ key: 'input', label: m.billing_input() },
		{ key: 'output', label: m.billing_output() },
		{ key: 'calls', label: m.billing_requests() },
		{ key: 'people', label: m.admin_usage_col_people() },
		{ key: 'spend', label: m.billing_spend() },
	];
	const embeddingColumns = [
		{ key: 'model', label: m.admin_usage_col_model() },
		{ key: 'tokens', label: m.admin_usage_col_tokens() },
		{ key: 'calls', label: m.billing_requests() },
		{ key: 'people', label: m.admin_usage_col_people() },
	];
</script>

<PageHeader title={m.admin_usage()} lede={m.admin_usage_lede()} agent="supervisor" />

{#if !usage}
	<p class="md-body-large" role="alert" style="color: var(--md-sys-color-error)">{m.admin_usage_load_failed()}</p>
{:else}
	<nav class="months" aria-label={m.money_month()}>
		<IconButton variant="filled-tonal" href="?month={shiftMonth(usage.month, -1)}" aria-label={m.money_prev_month()}><IconChevronLeft /></IconButton>
		<span class="months__current">{monthLabel(usage.month)}</span>
		<IconButton
			variant="filled-tonal"
			href={isCurrent ? undefined : `?month=${shiftMonth(usage.month, 1)}`}
			disabled={isCurrent}
			aria-label={m.money_next_month()}
		>
			<IconChevronRight />
		</IconButton>
		<span class="months__zone">UTC</span>
	</nav>

	<dl class="tiles">
		<div class="tile">
			<dt>{m.admin_usage_chat_tokens()}</dt>
			<dd title={formatTokensFull(usage.chat.tokens)}>{formatTokens(usage.chat.tokens)}</dd>
			<span class="tile__meta">{m.admin_usage_in_out({ input: formatTokens(usage.chat.input_tokens), output: formatTokens(usage.chat.output_tokens) })}</span>
		</div>
		<div class="tile">
			<dt>{m.billing_spend()}</dt>
			<dd><Money value={formatUsd(usage.chat.spend_usd)} /></dd>
			{#if usage.chat.own_key_tokens > 0}
				<span class="tile__meta">{m.admin_usage_own_key_tokens({ tokens: formatTokens(usage.chat.own_key_tokens) })}</span>
			{/if}
		</div>
		<div class="tile">
			<dt>{m.admin_usage_embedding_tokens()}</dt>
			<dd title={formatTokensFull(usage.embedding.tokens)}>{formatTokens(usage.embedding.tokens)}</dd>
			<span class="tile__meta">{m.admin_usage_requests({ count: formatTokensFull(usage.embedding.calls) })}</span>
		</div>
		<div class="tile">
			<dt>{m.billing_requests()}</dt>
			<dd>{formatTokensFull(usage.chat.calls)}</dd>
			<span class="tile__meta">{m.admin_usage_people({ count: formatTokensFull(usage.chat.people) })}</span>
		</div>
	</dl>

	{#if usage.chat.calls === 0 && usage.embedding.calls === 0}
		<p class="empty">{m.admin_usage_empty({ month: monthLabel(usage.month) })}</p>
	{:else}
		<section class="panel" aria-labelledby="day-heading">
			<div class="panel__head">
				<h2 id="day-heading" class="panel__title">{m.admin_usage_by_day()}</h2>
				<IconButton selected={showDayTable} aria-label={m.common_show_table()} onclick={() => (showDayTable = !showDayTable)}>
					<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="2" /><path d="M3 10h18M3 15h18M9 10v10" /></svg>
				</IconButton>
			</div>
			{#if showDayTable}
				<div class="table-scroll">
					<table class="table">
						<thead><tr><th scope="col">{m.money_day()}</th><th scope="col" class="num">{m.admin_usage_chat_tokens()}</th><th scope="col" class="num">{m.admin_usage_embedding_tokens()}</th></tr></thead>
						<tbody>
							{#each days.filter((d) => d.chat > 0 || d.embedding > 0) as d (d.date)}
								<tr><td>{dayLabel(d.date)}</td><td class="num">{formatTokensFull(d.chat)}</td><td class="num">{formatTokensFull(d.embedding)}</td></tr>
							{/each}
						</tbody>
					</table>
				</div>
			{:else}
				<div class="daychart">
					<svg viewBox="0 0 {CHART_W} {CHART_H}" class="daychart__svg" role="img" aria-label={m.admin_usage_day_chart({ month: monthLabel(usage.month) })}>
						{#each dayScale.ticks as tick (tick)}
							<line x1={AXIS_W} x2={CHART_W} y1={y(tick)} y2={y(tick)} class="daychart__grid" />
							<text x={AXIS_W - 8} y={y(tick) + 4} class="daychart__tick" text-anchor="end">{formatTokens(tick)}</text>
						{/each}
						{#each days as d (d.date)}
							{@const x = AXIS_W + (d.day - 1) * slot + (slot - barW) / 2}
							{#if d.chat > 0}
								<path d={barPath(x, barW, y(d.chat))} class="daychart__bar" class:daychart__bar--dim={hoverDay !== null && hoverDay !== d.day} />
							{/if}
							{#if d.day === 1 || d.day % 7 === 0}
								<text x={x + barW / 2} y={CHART_H - 4} class="daychart__tick" text-anchor="middle">{d.day}</text>
							{/if}
							<rect
								x={AXIS_W + (d.day - 1) * slot}
								y={TOP_Y}
								width={slot}
								height={BASE_Y - TOP_Y}
								class="daychart__hit"
								role="presentation"
								onpointerenter={() => (hoverDay = d.day)}
								onpointerleave={() => (hoverDay = null)}
							/>
						{/each}
					</svg>
					{#if hovered}
						<div class="tooltip" style="left: {((AXIS_W + (hovered.day - 0.5) * slot) / CHART_W) * 100}%; top: {(y(hovered.chat) / CHART_H) * 100}%">
							<span class="tooltip__title">{dayLabel(hovered.date)}</span>
							<span>{m.admin_usage_tip_chat({ tokens: formatTokensFull(hovered.chat) })}</span>
							<span>{m.admin_usage_tip_embedding({ tokens: formatTokensFull(hovered.embedding) })}</span>
						</div>
					{/if}
				</div>
			{/if}
		</section>

		<section class="section" aria-labelledby="models-heading">
			<div class="section__head">
				<h2 id="models-heading" class="panel__title">{m.admin_usage_chat_models()}</h2>
				<p class="section__lede">{m.admin_usage_chat_models_lede()}</p>
			</div>
			{#if usage.models.length === 0}
				<p class="empty">{m.admin_usage_no_chat()}</p>
			{:else}
				<DataTable card columns={modelColumns}>
					{#each usage.models as row ((row.admin_model_id ?? '') + row.provider + row.model_id + row.source)}
						<tr>
							<td>
								<div class="model-cell">
									<span class="model-cell__name">
										{row.label}
										{#if row.source === 'own_key'}<span class="chip">{m.admin_usage_own_key()}</span>{/if}
										{#if row.deleted}<span class="chip chip--muted">{m.admin_usage_deleted()}</span>{/if}
									</span>
									<span class="model-cell__meta">{row.provider} · {row.model_id || '—'}</span>
								</div>
							</td>
							<td>
								<div class="bar-cell">
									<span class="bar-cell__track"><span class="bar-cell__fill" style="width: {Math.max(2, (row.tokens / modelMax) * 100)}%"></span></span>
									<span class="num" title={formatTokensFull(row.tokens)}>{formatTokens(row.tokens)}</span>
								</div>
							</td>
							<td class="num" title={formatTokensFull(row.input_tokens)}>{formatTokens(row.input_tokens)}</td>
							<td class="num" title={formatTokensFull(row.output_tokens)}>{formatTokens(row.output_tokens)}</td>
							<td class="num">{formatTokensFull(row.calls)}</td>
							<td class="num">{formatTokensFull(row.people)}</td>
							<td class="num">{#if row.source === 'own_key'}{m.billing_not_billed()}{:else}<Money value={formatUsd(row.spend_usd)} />{/if}</td>
						</tr>
					{/each}
				</DataTable>
			{/if}
		</section>

		<section class="section" aria-labelledby="embedding-heading">
			<div class="section__head">
				<h2 id="embedding-heading" class="panel__title">{m.admin_usage_embedding_models()}</h2>
				<p class="section__lede">{m.admin_usage_embedding_lede()}</p>
			</div>
			{#if usage.embedding_models.length === 0}
				<p class="empty">{m.admin_usage_no_embedding()}</p>
			{:else}
				<DataTable card columns={embeddingColumns}>
					{#each usage.embedding_models as row (row.provider + row.model_id)}
						<tr>
							<td>
								<div class="model-cell">
									<span class="model-cell__name">
										{row.model_id || row.provider}
										{#if row.current}<span class="chip">{m.admin_usage_in_use()}</span>{/if}
									</span>
									<span class="model-cell__meta">{row.provider}</span>
								</div>
							</td>
							<td class="num">
								<span>
									<span title={formatTokensFull(row.tokens)}>{formatTokens(row.tokens)}</span>
									{#if row.estimated_calls > 0}<span class="estimate">{m.admin_usage_estimated()}</span>{/if}
								</span>
							</td>
							<td class="num">{formatTokensFull(row.calls)}</td>
							<td class="num">{formatTokensFull(row.people)}</td>
						</tr>
					{/each}
				</DataTable>
				{#if usage.embedding.estimated_calls > 0}
					<p class="note">{m.admin_usage_estimated_note()}</p>
				{/if}
			{/if}
		</section>
	{/if}
{/if}

<style>
	.months {
		display: inline-flex;
		align-items: center;
		align-self: flex-start;
		gap: 4px;
		padding: 4px 14px 4px 4px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface);
	}
	.months__current {
		min-width: 140px;
		text-align: center;
		font-weight: 650;
	}
	.months__zone {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.tiles {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(170px, 1fr));
		gap: 12px;
		margin: 0;
	}
	.tile {
		display: flex;
		flex-direction: column;
		gap: 4px;
		min-width: 0;
		padding: 18px 20px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		container-type: inline-size;
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.tile dt {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.tile dd {
		margin: 0;
		font-size: clamp(1.125rem, 15cqi, 1.75rem);
		font-weight: 650;
		line-height: 1.2;
		overflow-wrap: anywhere;
	}
	.tile__meta {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.panel {
		display: flex;
		flex-direction: column;
		gap: 14px;
		min-width: 0;
		padding: 22px 24px;
		border-radius: var(--md-sys-shape-corner-extra-large-increased);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.panel__head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}
	.panel__title {
		margin: 0;
		font-size: 1.125rem;
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
	}
	.section {
		display: flex;
		flex-direction: column;
		gap: 10px;
		min-width: 0;
	}
	.section__lede {
		margin: 2px 0 0;
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.empty {
		margin: 0;
		padding: 20px 24px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface-variant);
	}
	.note {
		margin: 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.model-cell {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 180px;
	}
	.model-cell__name {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px;
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}
	.model-cell__meta {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
		overflow-wrap: anywhere;
	}
	.chip {
		padding: 1px 8px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font-size: 0.6875rem;
		font-weight: 700;
	}
	.chip--muted {
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
	}
	.bar-cell {
		min-width: 160px;
	}
	.bar-cell,
	.bar-cell__track {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.bar-cell__track {
		flex: 1;
		min-width: 60px;
	}
	.bar-cell__fill {
		height: 12px;
		border-radius: 0 4px 4px 0;
		background: var(--nomi-chart);
	}
	.num {
		text-align: right;
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}
	.estimate {
		margin-left: 6px;
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.daychart {
		position: relative;
	}
	.daychart__svg {
		display: block;
		width: 100%;
		height: auto;
		overflow: visible;
	}
	.daychart__grid {
		stroke: var(--md-sys-color-outline-variant);
		stroke-width: 1;
		vector-effect: non-scaling-stroke;
	}
	.daychart__tick {
		font-size: 15px;
		fill: var(--md-sys-color-on-surface-variant);
		font-variant-numeric: tabular-nums;
	}
	.daychart__bar {
		fill: var(--nomi-chart);
		transition: opacity var(--nomi-motion-effects-fast);
	}
	.daychart__bar--dim {
		opacity: 0.35;
	}
	.daychart__hit {
		fill: transparent;
	}
	.tooltip {
		position: absolute;
		z-index: 2;
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 8px 12px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-inverse-surface);
		color: var(--md-sys-color-inverse-on-surface);
		font-size: 0.8125rem;
		white-space: nowrap;
		pointer-events: none;
		translate: -50% calc(-100% - 8px);
	}
	.tooltip__title {
		font-weight: 650;
	}
	.table-scroll {
		max-height: 280px;
		overflow-y: auto;
	}
	.table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.875rem;
	}
	.table th {
		padding: 8px 6px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		font-weight: 600;
		text-align: left;
		color: var(--md-sys-color-on-surface-variant);
	}
	.table th.num {
		text-align: right;
	}
	.table td {
		padding: 8px 6px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	@media (max-width: 600px) {
		.daychart {
			overflow-x: auto;
			margin-top: -64px;
			padding-top: 64px;
		}
		.daychart__svg {
			min-width: 540px;
		}
		.panel {
			padding: 18px 16px;
		}
	}
</style>
