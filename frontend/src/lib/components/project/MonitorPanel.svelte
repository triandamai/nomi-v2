<script lang="ts">
	// The published app while it runs: whether it's up, CPU and memory of each container (sampled
	// every few seconds while this is open), what Shipyard reported, and what the app prints.
	// Visitors, requests and history wait for Shipyard to report them (docs/shipyard-todo.md):
	// nothing here is shown that Nomi can't back.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import StatusSquare from '$lib/components/project/StatusSquare.svelte';
	import { chartPaths, eventTone, filterLogs, liveSince, logLevel, orderContainers, peak, type LogFilter } from '$lib/monitorView';
	import { ago, bytes, projectApi } from '$lib/projectApi';
	import type { ProjectDeploy } from '$lib/projectDeploy.svelte';
	import type { ContainerUsage, ShipyardEvent } from '$lib/types';

	let { projectId, published, deploy = null }: { projectId: string; published: boolean; deploy?: ProjectDeploy | null } = $props();

	const locale = getLocale();
	/** Samples kept per container: about three minutes at one every few seconds. */
	const SAMPLES = 40;
	const SAMPLE_MS = 5000;
	const LOG_MS = 5000;
	const LOG_TAIL = 400;
	const EVENTS_SHOWN = 20;
	const CHART_WIDTH = 300;
	const CHART_HEIGHT = 80;
	/** Shipyard events worth a line here. */
	const EVENTS = ['service.crashed', 'service.resource_alert', 'deployment.failed', 'publish.failed', 'publish.succeeded'];

	const services = $derived(deploy?.overview?.live?.services ?? []);
	const upSince = $derived(liveSince(services));

	let containers = $state<ContainerUsage[]>([]);
	// Recent CPU and memory per container, oldest first.
	let history = $state<Record<string, { cpu: number[]; memory: number[] }>>({});
	let usageError = $state<string | null>(null);
	const shownContainers = $derived(orderContainers(containers));

	async function sample() {
		try {
			const usage = await projectApi<{ containers: ContainerUsage[] }>(projectId, 'deploy/usage');
			containers = usage.containers;
			const next = { ...history };
			for (const c of usage.containers) {
				if (!c.stats) continue;
				const h = next[c.container_id] ?? { cpu: [], memory: [] };
				next[c.container_id] = {
					cpu: [...h.cpu, c.stats.cpu_percent].slice(-SAMPLES),
					memory: [...h.memory, c.stats.memory_percent].slice(-SAMPLES),
				};
			}
			history = next;
			usageError = null;
		} catch (e) {
			usageError = (e as Error).message;
		}
	}

	let lines = $state<string[]>([]);
	let logsNote = $state<string | null>(null);
	let follow = $state(true);
	let level = $state<LogFilter>('all');
	let query = $state('');
	let logBox = $state<HTMLPreElement | null>(null);
	const shownLines = $derived(filterLogs(lines, level, query));

	async function loadLogs() {
		try {
			const logs = await projectApi<{ lines: string[]; note?: string }>(projectId, `deploy/logs?tail=${LOG_TAIL}`);
			lines = logs.lines;
			logsNote = logs.note ?? null;
			if (follow && logBox) requestAnimationFrame(() => logBox && (logBox.scrollTop = logBox.scrollHeight));
		} catch (e) {
			logsNote = (e as Error).message;
		}
	}

	let events = $state<ShipyardEvent[]>([]);
	async function loadEvents() {
		const all = await projectApi<ShipyardEvent[]>(projectId, 'deploy/events?limit=200').catch(() => []);
		events = all.filter((e) => EVENTS.includes(e.type)).slice(0, EVENTS_SHOWN);
	}

	$effect(() => {
		if (!published) return;
		void sample();
		void loadLogs();
		void loadEvents();
		const usageTimer = setInterval(() => !document.hidden && void sample(), SAMPLE_MS);
		const logTimer = setInterval(() => follow && !document.hidden && void loadLogs(), LOG_MS);
		return () => {
			clearInterval(usageTimer);
			clearInterval(logTimer);
		};
	});

	function percent(value: number): string {
		return `${new Intl.NumberFormat(locale, { maximumFractionDigits: value < 10 ? 1 : 0 }).format(value)}%`;
	}

	function eventText(event: ShipyardEvent): string {
		const data = event.data as Record<string, unknown>;
		switch (event.type) {
			case 'service.crashed':
				return m.pj_alert_crashed({ code: String(data.exit_code ?? '?') });
			case 'service.resource_alert':
				return m.pj_alert_resource({
					metric: String(data.metric ?? 'cpu'),
					value: typeof data.value === 'number' ? percent(data.value) : '?',
					threshold: typeof data.threshold === 'number' ? percent(data.threshold) : '?',
				});
			case 'publish.succeeded':
				return m.pj_mon_event_live();
			default:
				return m.pj_alert_failed({ error: String(data.error ?? '') });
		}
	}

	const FILTERS: { value: LogFilter; label: () => string }[] = [
		{ value: 'all', label: m.pj_mon_all },
		{ value: 'warn', label: m.pj_mon_warnings },
		{ value: 'error', label: m.pj_mon_errors },
	];
</script>

{#if !published}
	<p class="muted">{m.pj_monitor_unpublished()}</p>
{:else}
	<section class="card health" aria-label={m.pj_mon_health()}>
		<StatusSquare status={services.length === 0 ? 'pending' : services.some((s) => s.running < s.replicas) ? 'failed' : 'succeeded'} />
		<div class="health__text">
			<h3 class="health__title">
				{#if upSince}{m.pj_mon_up_since({ when: ago(upSince, locale) })}{:else}{m.pj_no_containers()}{/if}
			</h3>
			{#each services as service (service.service_id)}
				<p class="meta">{m.pj_mon_replicas({ service: service.name, running: service.running, replicas: service.replicas })}</p>
			{/each}
		</div>
	</section>

	<p class="later">{m.pj_mon_metrics_later()}</p>

	{#if usageError}<p class="error" role="alert">{usageError}</p>{/if}
	{#if shownContainers.length === 0 && !usageError}
		<p class="muted">{m.pj_no_containers()}</p>
	{/if}
	{#each shownContainers as c (c.container_id)}
		{@const h = history[c.container_id]}
		{@const cpu = h ? chartPaths(h.cpu, CHART_WIDTH, CHART_HEIGHT) : null}
		{@const memory = h ? chartPaths(h.memory, CHART_WIDTH, CHART_HEIGHT) : null}
		<section class="resources">
			<h3 class="section-title">{c.service}</h3>
			{#if c.stats}
				<div class="charts">
					<article class="card chart">
						<div class="chart__head">
							<span class="chart__title">{m.pj_cpu()}</span>
							<span class="meta">{m.pj_mon_now_peak({ now: percent(c.stats.cpu_percent), peak: percent(peak(h?.cpu ?? []) ?? c.stats.cpu_percent) })}</span>
						</div>
						<svg class="chart__svg" viewBox="0 0 {CHART_WIDTH} {CHART_HEIGHT}" preserveAspectRatio="none" aria-hidden="true">
							<line class="chart__base" x1="0" y1={CHART_HEIGHT} x2={CHART_WIDTH} y2={CHART_HEIGHT} />
							{#if cpu}<path class="chart__area" d={cpu.area} /><path class="chart__line" d={cpu.line} />{/if}
						</svg>
						<span class="caption">{m.pj_mon_since_open()}{#if c.limits.cpu}&nbsp;· {m.pj_cpu_limit({ cpu: c.limits.cpu })}{/if}</span>
					</article>
					<article class="card chart">
						<div class="chart__head">
							<span class="chart__title">{m.pj_memory()}</span>
							<span class="meta">{m.pj_mon_memory_now({ used: bytes(c.stats.memory_usage_bytes, locale), limit: bytes(c.stats.memory_limit_bytes, locale) })}</span>
						</div>
						<svg class="chart__svg" viewBox="0 0 {CHART_WIDTH} {CHART_HEIGHT}" preserveAspectRatio="none" aria-hidden="true">
							<line class="chart__base" x1="0" y1={CHART_HEIGHT} x2={CHART_WIDTH} y2={CHART_HEIGHT} />
							{#if memory}<path class="chart__area" d={memory.area} /><path class="chart__line" d={memory.line} />{/if}
						</svg>
						<span class="caption">{m.pj_mon_since_open()} · {m.pj_mon_now_peak({ now: percent(c.stats.memory_percent), peak: percent(peak(h?.memory ?? []) ?? c.stats.memory_percent) })}</span>
					</article>
				</div>
			{:else}
				<p class="muted">{m.pj_stats_unavailable()}</p>
			{/if}
		</section>
	{/each}

	{#if events.length > 0}
		<h3 class="section-title">{m.pj_mon_events()}</h3>
		<ul class="card list">
			{#each events as event (event.id)}
				<li class="event">
					<StatusSquare status={eventTone(event.type)} size={28} />
					<span class="event__text">{eventText(event)}</span>
					<span class="meta">{event.service ? `${event.service} · ` : ''}{ago(event.created_at, locale)}</span>
				</li>
			{/each}
		</ul>
	{/if}

	<section class="card logs">
		<div class="logs__head">
			<h3 class="section-title logs__title">{m.pj_mon_logs()}</h3>
			<div class="segmented" role="radiogroup" aria-label={m.pj_mon_log_level()}>
				{#each FILTERS as filter (filter.value)}
					<button type="button" role="radio" aria-checked={level === filter.value} onclick={() => (level = filter.value)}>{filter.label()}</button>
				{/each}
			</div>
		</div>
		<div class="logs__tools">
			<input class="search" type="search" placeholder={m.pj_mon_search()} aria-label={m.pj_mon_search()} bind:value={query} />
			<label class="follow"><input type="checkbox" bind:checked={follow} />{m.pj_mon_follow()}</label>
		</div>
		{#if logsNote}<p class="muted">{logsNote}</p>{/if}
		<pre class="log" bind:this={logBox}>{#if lines.length === 0}{m.pj_log_empty()}{:else if shownLines.length === 0}{m.pj_mon_no_matches()}{:else}{#each shownLines as line, i (i)}<span class="log__line log__line--{logLevel(line)}">{line}
</span>{/each}{/if}</pre>
	</section>
{/if}

<style>
	.card {
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.muted {
		margin: 8px 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.error {
		margin: 8px 0;
		color: var(--md-sys-color-error);
		font-size: 0.875rem;
	}
	.meta,
	.caption {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.caption {
		font-size: 0.75rem;
	}
	.section-title {
		margin: 20px 0 8px;
		color: var(--md-sys-color-on-surface);
		font-size: 0.9375rem;
		font-weight: 700;
	}
	.health {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 16px 18px;
	}
	.health__text {
		min-width: 0;
	}
	.health__title {
		margin: 0 0 2px;
		color: var(--md-sys-color-on-surface);
		font-size: 1rem;
		font-weight: 600;
	}
	.later {
		margin: 12px 0 0;
		padding: 10px 14px;
		border: 1px dashed var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.charts {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 12px;
	}
	@media (max-width: 600px) {
		.charts {
			grid-template-columns: 1fr;
		}
	}
	.chart {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 14px 16px;
	}
	.chart__head {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 4px 10px;
	}
	.chart__title {
		color: var(--md-sys-color-on-surface);
		font-size: 0.9375rem;
		font-weight: 600;
	}
	.chart__svg {
		width: 100%;
		height: 96px;
	}
	.chart__base {
		stroke: var(--md-sys-color-outline-variant);
		stroke-width: 1;
		vector-effect: non-scaling-stroke;
	}
	.chart__line {
		fill: none;
		stroke: var(--nomi-chart, var(--md-sys-color-primary));
		stroke-width: 2;
		vector-effect: non-scaling-stroke;
	}
	.chart__area {
		fill: var(--nomi-chart, var(--md-sys-color-primary));
		opacity: 0.14;
	}
	.list {
		margin: 0;
		padding: 0 16px;
		list-style: none;
	}
	.event {
		display: grid;
		grid-template-columns: auto 1fr auto;
		gap: 12px;
		align-items: center;
		min-height: 52px;
		padding: 8px 0;
		border-top: 1px solid var(--md-sys-color-surface-container-low);
		font-size: 0.875rem;
	}
	.event:first-child {
		border-top: 0;
	}
	.event__text {
		min-width: 0;
		color: var(--md-sys-color-on-surface);
		overflow-wrap: anywhere;
	}
	.logs {
		margin-top: 20px;
		padding: 14px 16px 16px;
	}
	.logs__head {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 8px 12px;
	}
	.logs__title {
		margin: 0;
	}
	.segmented {
		display: flex;
		gap: 2px;
		padding: 3px;
		border-radius: 999px;
		background: var(--md-sys-color-surface-container-low);
	}
	.segmented button {
		min-height: 32px;
		padding: 4px 12px;
		border: 0;
		border-radius: 999px;
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		font: 500 0.8125rem var(--md-ref-typeface-plain);
		cursor: pointer;
	}
	.segmented button:hover {
		color: var(--md-sys-color-on-surface);
	}
	.segmented button:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 1px;
	}
	.segmented button[aria-checked='true'] {
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		font-weight: 600;
	}
	.logs__tools {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px 14px;
		margin: 12px 0 10px;
	}
	.search {
		flex: 1;
		min-width: 180px;
		min-height: 40px;
		padding: 8px 14px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 999px;
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: 0.875rem var(--md-ref-typeface-plain);
	}
	.search:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 1px;
	}
	.follow {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		min-height: 44px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		cursor: pointer;
	}
	.log {
		height: min(48vh, 460px);
		margin: 0;
		padding: 12px 14px;
		overflow: auto;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface);
		font: 0.78rem/1.55 var(--md-ref-typeface-mono);
		white-space: pre-wrap;
		overflow-wrap: anywhere;
	}
	.log__line--error {
		color: var(--md-sys-color-error);
	}
	.log__line--warn {
		color: var(--md-sys-color-tertiary);
	}
</style>
