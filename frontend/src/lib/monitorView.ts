// What the Monitor sheet works out from what Shipyard gives Nomi: log levels (Shipyard's runtime
// log lines are plain text), peaks and chart paths for CPU and memory, traffic history with its
// gaps (a bucket Shipyard didn't measure is a gap, never a 0), and which service says how long
// the app has been up.

import type { ContainerUsage, MetricsBucket, ShipyardService } from '$lib/types';

export type LogLevel = 'error' | 'warn' | 'info';
export type LogFilter = 'all' | 'warn' | 'error';

const ERROR_WORDS = /\b(error|fatal|panic|exception)\b/i;
const WARN_WORDS = /\b(warn|warning)\b/i;
/** Charts never scale below 100% so a quiet app doesn't look busy. */
const MIN_CHART_MAX = 100;

export function logLevel(line: string): LogLevel {
	if (ERROR_WORDS.test(line)) return 'error';
	if (WARN_WORDS.test(line)) return 'warn';
	return 'info';
}

export function filterLogs(lines: string[], level: LogFilter, query: string): string[] {
	const needle = query.trim().toLowerCase();
	return lines.filter((line) => {
		const lineLevel = logLevel(line);
		if (level === 'error' && lineLevel !== 'error') return false;
		if (level === 'warn' && lineLevel === 'info') return false;
		return !needle || line.toLowerCase().includes(needle);
	});
}

export function peak(values: number[]): number | null {
	return values.length ? Math.max(...values) : null;
}

/** SVG paths for a line and the area under it, in a `width` × `height` box; null under 2 samples. */
export function chartPaths(values: number[], width: number, height: number, max = Math.max(MIN_CHART_MAX, ...values)): { line: string; area: string } | null {
	if (values.length < 2) return null;
	const round = (n: number) => Math.round(n * 10) / 10;
	const points = values.map((v, i) => `${round((i / (values.length - 1)) * width)},${round(height - (Math.min(v, max) / max) * height)}`);
	const line = `M${points.join(' L')}`;
	return { line, area: `${line} L${width},${height} L0,${height} Z` };
}

/** How an event reads: something broke, something to watch, or something that went well. */
export function eventTone(type: string): 'failed' | 'running' | 'succeeded' {
	if (type.endsWith('crashed') || type.endsWith('failed')) return 'failed';
	if (type.endsWith('succeeded')) return 'succeeded';
	return 'running';
}

const isDatabase = (name: string, type?: string) => name === 'db' || type === 'database';

/** When the app's web service last went up, as Shipyard says. */
export function liveSince(services: ShipyardService[]): string | null {
	const web = services.find((s) => !isDatabase(s.name, s.type)) ?? services[0];
	return web?.last_deployment?.created_at ?? null;
}

/** The app's own containers first, the database last. */
export function orderContainers(containers: ContainerUsage[]): ContainerUsage[] {
	return [...containers].sort((a, b) => Number(isDatabase(a.service)) - Number(isDatabase(b.service)));
}

/** One stretch of measured values, drawn from `x` along a chart `width` wide. */
export interface ChartRun {
	x: number;
	line: string;
	area: string;
}

/**
 * Chart paths for a series with unmeasured (null) points: each unbroken stretch of measured values
 * is drawn on its own at its place in time, so a gap stays a gap. Lone points aren't drawn.
 */
export function gappedChartPaths(values: (number | null)[], width: number, height: number, max: number): ChartRun[] {
	const runs: ChartRun[] = [];
	const step = values.length > 1 ? width / (values.length - 1) : 0;
	let start = 0;
	while (start < values.length) {
		if (values[start] === null) {
			start++;
			continue;
		}
		let end = start;
		while (end + 1 < values.length && values[end + 1] !== null) end++;
		const run = values.slice(start, end + 1) as number[];
		const paths = chartPaths(run, (run.length - 1) * step, height, max);
		if (paths) runs.push({ x: Math.round(start * step * 10) / 10, ...paths });
		start = end + 1;
	}
	return runs;
}

/** The top of a counter chart: its largest measured value, at least 1 so a flat zero still draws. */
export function countMax(values: (number | null)[]): number {
	return Math.max(1, ...values.filter((v): v is number => v !== null));
}

/** Percent change from the previous range; null when either wasn't measured or there was nothing before. */
export function change(now: number | null, before: number | null): number | null {
	if (now === null || before === null) return null;
	if (before === 0) return now === 0 ? 0 : null;
	return ((now - before) / before) * 100;
}

/** CPU and memory % per service over the buckets, averaged across its replicas; null where unmeasured. */
export function serviceHistory(buckets: MetricsBucket[]): Record<string, { cpu: (number | null)[]; memory: (number | null)[] }> {
	const services = new Set(buckets.flatMap((b) => b.containers.map((c) => c.service)));
	const mean = (values: (number | null)[]) => {
		const measured = values.filter((v): v is number => v !== null);
		return measured.length ? measured.reduce((a, b) => a + b, 0) / measured.length : null;
	};
	const history: Record<string, { cpu: (number | null)[]; memory: (number | null)[] }> = {};
	for (const service of services) {
		const of = (b: MetricsBucket) => b.containers.filter((c) => c.service === service);
		history[service] = {
			cpu: buckets.map((b) => mean(of(b).map((c) => c.cpu_percent))),
			memory: buckets.map((b) => mean(of(b).map((c) => c.memory_percent))),
		};
	}
	return history;
}
