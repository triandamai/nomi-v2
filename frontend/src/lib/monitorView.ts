// What the Monitor sheet works out from what Shipyard gives Nomi today: log levels (Shipyard's
// runtime log lines are plain text), peaks and chart paths for the CPU and memory sampled while
// the sheet is open, and which service says how long the app has been up.

import type { ContainerUsage, ShipyardService } from '$lib/types';

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
