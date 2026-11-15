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

const percent = (share: number) => `${new Intl.NumberFormat('en-US', { maximumFractionDigits: 1 }).format(share * 100)}%`;

/** How long since `since`: "2h 37m", "2d", "45m". */
function lasted(since: string, now: Date): string {
	const minutes = Math.max(0, Math.floor((now.getTime() - Date.parse(since)) / 60_000));
	if (minutes >= 48 * 60) return `${Math.floor(minutes / (24 * 60))}d`;
	if (minutes >= 60) return `${Math.floor(minutes / 60)}h ${minutes % 60}m`;
	return `${minutes}m`;
}

/**
 * A site's own numbers (Landing page WUiji, step 5): forms sent with their share of visits, and
 * uptime with how long it's been looked at. Not measured stays null, never 0.
 */
export function siteStats(metrics: ProjectMetrics, now: Date) {
	const sends = metrics.form_sends ?? null;
	const visits = metrics.totals.visitors;
	return {
		formSends: sends === null ? null : whole(sends),
		formShare: sends === null || !visits ? null : percent(sends / visits),
		uptime: metrics.uptime ? percent(metrics.uptime.share) : null,
		uptimeFor: metrics.uptime ? lasted(metrics.uptime.since, now) : null,
	};
}

/** Where a site's visits came from, the largest first, each as a share; null until any were reported. */
export function sourceRows(sources: { name: string; share: number }[] | null | undefined): { name: string; value: string }[] | null {
	if (!sources) return null;
	return [...sources].sort((a, b) => b.share - a.share).map((source) => ({ name: source.name, value: `${Math.round(source.share * 100)}%` }));
}

export type TrafficBar = { at: string; share: number | null; count: number | null };

/**
 * Each bucket's requests (or, for a site, its visits) against the busiest one; an unmeasured bucket
 * stays null (a gap).
 */
export function trafficBars(buckets: MetricsBucket[], timeZone?: string, measure: 'requests' | 'visitors' = 'requests'): { bars: TrafficBar[]; peak: { at: string; count: number } | null } {
	const hour = (iso: string) => new Intl.DateTimeFormat('en-GB', { timeZone, hour: '2-digit', minute: '2-digit', hour12: false }).format(new Date(iso));
	const most = Math.max(0, ...buckets.map((bucket) => bucket[measure] ?? 0));
	const bars = buckets.map((bucket) => ({ at: hour(bucket.at), share: bucket[measure] === null ? null : most ? bucket[measure] / most : 0, count: bucket[measure] }));
	const top = bars.find((bar) => bar.count !== null && bar.count === most && most > 0);
	return { bars, peak: top ? { at: top.at, count: most } : null };
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
