<script lang="ts">
	// The published app while it runs: CPU and memory of each container (sampled every few
	// seconds while this is open), what the app prints, and what Shipyard warned about.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import Button from '$lib/components/m3/Button.svelte';
	import IconRefresh from '$lib/components/icons/IconRefresh.svelte';
	import { ago, bytes, projectApi } from '$lib/projectApi';
	import type { ContainerUsage, ShipyardEvent } from '$lib/types';

	let { projectId, published }: { projectId: string; published: boolean } = $props();

	const locale = getLocale();
	const SAMPLES = 40;

	let containers = $state<ContainerUsage[]>([]);
	// Recent CPU and memory per container, oldest first.
	let history = $state<Record<string, { cpu: number[]; memory: number[] }>>({});
	let usageError = $state<string | null>(null);

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
	let logsError = $state<string | null>(null);
	let follow = $state(true);
	let logBox = $state<HTMLPreElement | null>(null);

	async function loadLogs() {
		try {
			const logs = await projectApi<{ lines: string[]; note?: string }>(projectId, 'deploy/logs?tail=400');
			lines = logs.lines;
			logsError = logs.note ?? null;
			if (follow && logBox) requestAnimationFrame(() => logBox && (logBox.scrollTop = logBox.scrollHeight));
		} catch (e) {
			logsError = (e as Error).message;
		}
	}

	let events = $state<ShipyardEvent[]>([]);
	const ALERTS = ['service.crashed', 'service.resource_alert', 'deployment.failed', 'publish.failed'];
	async function loadEvents() {
		const all = await projectApi<ShipyardEvent[]>(projectId, 'deploy/events?limit=200').catch(() => []);
		events = all.filter((e) => ALERTS.includes(e.type)).slice(0, 20);
	}

	$effect(() => {
		if (!published) return;
		void sample();
		void loadLogs();
		void loadEvents();
		const usageTimer = setInterval(() => !document.hidden && void sample(), 5000);
		const logTimer = setInterval(() => follow && !document.hidden && void loadLogs(), 5000);
		return () => {
			clearInterval(usageTimer);
			clearInterval(logTimer);
		};
	});

	function sparkline(values: number[]): string {
		if (values.length < 2) return '';
		const max = Math.max(100, ...values);
		return values.map((v, i) => `${((i / (SAMPLES - 1)) * 100).toFixed(1)},${(30 - (v / max) * 28).toFixed(1)}`).join(' ');
	}

	function percent(value: number): string {
		return `${new Intl.NumberFormat(locale, { maximumFractionDigits: value < 10 ? 1 : 0 }).format(value)}%`;
	}

	function alertText(event: ShipyardEvent): string {
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
			default:
				return m.pj_alert_failed({ error: String(data.error ?? '') });
		}
	}
</script>

{#if !published}
	<p class="muted">{m.pj_monitor_unpublished()}</p>
{:else}
	<h3 class="section-title">{m.pj_resources()}</h3>
	{#if usageError}<p class="error">{usageError}</p>{/if}
	{#if containers.length === 0 && !usageError}
		<p class="muted">{m.pj_no_containers()}</p>
	{/if}
	<div class="containers">
		{#each containers as c (c.container_id)}
			{@const h = history[c.container_id]}
			<article class="container">
				<header class="container__head">
					<span class="container__name">{c.service}</span>
					<span class="container__id">{c.container_id.slice(0, 12)}</span>
				</header>
				{#if c.stats}
					<div class="meters">
						<div class="meter">
							<div class="meter__label"><span>{m.pj_cpu()}</span><span class="meter__value">{percent(c.stats.cpu_percent)}</span></div>
							<svg class="spark" viewBox="0 0 100 30" preserveAspectRatio="none" aria-hidden="true">
								<polyline points={sparkline(h?.cpu ?? [])} />
							</svg>
							{#if c.limits.cpu}<span class="meter__limit">{m.pj_cpu_limit({ cpu: c.limits.cpu })}</span>{/if}
						</div>
						<div class="meter">
							<div class="meter__label">
								<span>{m.pj_memory()}</span>
								<span class="meter__value">{bytes(c.stats.memory_usage_bytes, locale)} / {bytes(c.stats.memory_limit_bytes, locale)}</span>
							</div>
							<div class="bar" role="progressbar" aria-label={m.pj_memory()} aria-valuenow={Math.round(c.stats.memory_percent)} aria-valuemin={0} aria-valuemax={100}>
								<span class="bar__fill" class:bar__fill--high={c.stats.memory_percent > 85} style="width: {Math.min(100, c.stats.memory_percent)}%"></span>
							</div>
						</div>
					</div>
					<p class="container__io">
						{m.pj_network({ rx: bytes(c.stats.net_rx_bytes, locale), tx: bytes(c.stats.net_tx_bytes, locale) })} ·
						{m.pj_disk({ read: bytes(c.stats.block_read_bytes, locale), write: bytes(c.stats.block_write_bytes, locale) })}
					</p>
				{:else}
					<p class="muted">{m.pj_stats_unavailable()}</p>
				{/if}
			</article>
		{/each}
	</div>

	{#if events.length > 0}
		<h3 class="section-title">{m.pj_alerts()}</h3>
		<ul class="alerts">
			{#each events as event (event.id)}
				<li class="alert">
					<span class="alert__text">{alertText(event)}</span>
					<span class="alert__when">{event.service ?? ''} · {ago(event.created_at, locale)}</span>
				</li>
			{/each}
		</ul>
	{/if}

	<div class="logs-head">
		<h3 class="section-title">{m.pj_runtime_logs()}</h3>
		<label class="follow"><input type="checkbox" bind:checked={follow} />{m.pj_follow()}</label>
		<Button variant="text" size="s" onclick={loadLogs}><IconRefresh size={16} />{m.pj_refresh()}</Button>
	</div>
	{#if logsError}<p class="muted">{logsError}</p>{/if}
	<pre class="log" bind:this={logBox}>{lines.length ? lines.join('\n') : m.pj_log_empty()}</pre>
{/if}

<style>
	.section-title {
		margin: 20px 0 8px;
		color: var(--md-sys-color-on-surface);
		font-size: 0.9375rem;
		font-weight: 700;
	}
	.section-title:first-child {
		margin-top: 0;
	}
	.muted {
		margin: 8px 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.error {
		margin: 8px 0;
		color: var(--md-sys-color-error);
	}
	.containers {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
		gap: 12px;
	}
	.container {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 14px 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container);
	}
	.container__head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 8px;
	}
	.container__name {
		font-weight: 700;
	}
	.container__id {
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
	}
	.meters {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.meter__label {
		display: flex;
		justify-content: space-between;
		gap: 8px;
		margin-bottom: 4px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.meter__value {
		color: var(--md-sys-color-on-surface);
		font-variant-numeric: tabular-nums;
		font-weight: 600;
	}
	.meter__limit {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.75rem;
	}
	.spark {
		display: block;
		width: 100%;
		height: 36px;
		border-radius: 4px;
		background: var(--md-sys-color-surface-container-high);
	}
	.spark polyline {
		fill: none;
		stroke: var(--nomi-chart, var(--md-sys-color-primary));
		stroke-width: 1.6;
		vector-effect: non-scaling-stroke;
	}
	.bar {
		height: 8px;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-highest);
	}
	.bar__fill {
		display: block;
		height: 100%;
		border-radius: inherit;
		background: var(--nomi-chart, var(--md-sys-color-primary));
		transition: width 400ms ease-out;
	}
	.bar__fill--high {
		background: var(--md-sys-color-error);
	}
	.container__io {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.75rem;
	}
	.alerts {
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.alert {
		display: flex;
		flex-wrap: wrap;
		justify-content: space-between;
		gap: 4px 12px;
		padding: 10px 0;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		font-size: 0.875rem;
	}
	.alert__when {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.logs-head {
		display: flex;
		align-items: center;
		gap: 8px;
		margin-top: 12px;
	}
	.logs-head .section-title {
		margin: 0 auto 0 0;
	}
	.follow {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		min-height: 44px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
		cursor: pointer;
	}
	.follow input {
		accent-color: var(--md-sys-color-primary);
	}
	.log {
		height: min(52vh, 520px);
		margin: 8px 0 0;
		padding: 12px 14px;
		overflow: auto;
		border-radius: 8px;
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		line-height: 1.55;
		white-space: pre-wrap;
		word-break: break-word;
	}
</style>
