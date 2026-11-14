import type { ContainerUsage, MetricsBucket, ProjectMetrics } from '$lib/adapters/api';

// Monitor (Flow / Build — Workspace tools U16wU, step 10): the published app's numbers as Shipyard
// measured them, worded for the sheet. Whatever wasn't measured is null (the sheet says "Not
// measured"), never 0. The pen's uptime isn't measured by Shipyard: visitors stand in its place.

const whole = (value: number) => new Intl.NumberFormat('en-US', { maximumFractionDigits: 0 }).format(value);

export function monitorStats(metrics: ProjectMetrics) {
	const { requests, errors_5xx: errors, p95_ms: p95, visitors } = metrics.totals;
	return {
		requests: requests === null ? null : whole(requests),
		errors: errors === null ? null : whole(errors),
		errorShare: errors === null || !requests ? null : `${new Intl.NumberFormat('en-US', { maximumFractionDigits: 1 }).format((errors / requests) * 100)}%`,
		p95: p95 === null ? null : `${whole(p95)} ms`,
		visitors: visitors === null ? null : whole(visitors),
	};
}

export type TrafficBar = { at: string; share: number | null; requests: number | null };

/** Each bucket's requests against the busiest one; an unmeasured bucket stays null (a gap). */
export function trafficBars(buckets: MetricsBucket[], timeZone?: string): { bars: TrafficBar[]; peak: { at: string; requests: number } | null } {
	const hour = (iso: string) => new Intl.DateTimeFormat('en-GB', { timeZone, hour: '2-digit', minute: '2-digit', hour12: false }).format(new Date(iso));
	const most = Math.max(0, ...buckets.map((bucket) => bucket.requests ?? 0));
	const bars = buckets.map((bucket) => ({ at: hour(bucket.at), share: bucket.requests === null ? null : most ? bucket.requests / most : 0, requests: bucket.requests }));
	const top = bars.find((bar) => bar.requests !== null && bar.requests === most && most > 0);
	return { bars, peak: top ? { at: top.at, requests: most } : null };
}

const megabytes = (bytes: number) => Math.round(bytes / (1024 * 1024));

/** The running app's containers added up. */
export function resourceRows(usage: ContainerUsage[]) {
	const measured = usage.flatMap((container) => (container.stats ? [container.stats] : []));
	if (!measured.length) return { memory: null, cpu: null, bandwidth: null };
	const sum = (pick: (stats: (typeof measured)[number]) => number) => measured.reduce((total, stats) => total + pick(stats), 0);
	return {
		memory: `${megabytes(sum((stats) => stats.memory_usage_bytes))} / ${megabytes(sum((stats) => stats.memory_limit_bytes))} MB`,
		cpu: `${new Intl.NumberFormat('en-US', { maximumFractionDigits: 1 }).format(sum((stats) => stats.cpu_percent))}%`,
		bandwidth: `${megabytes(sum((stats) => stats.net_rx_bytes + stats.net_tx_bytes))} MB`,
	};
}
