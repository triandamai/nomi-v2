import { describe, expect, it } from 'vitest';
import type { ContainerUsage, MetricsBucket, ProjectMetrics } from '$lib/adapters/api';
import { monitorStats, resourceRows, trafficBars } from './monitor';

const counters = (extra: Partial<MetricsBucket> = {}) => ({ visitors: null, requests: null, errors_5xx: null, errors_4xx: null, p50_ms: null, p95_ms: null, ...extra });
const metrics = (extra: Partial<ProjectMetrics> = {}): ProjectMetrics => ({ range: '24h', step: '1h', buckets: [], totals: counters(), previous_totals: counters(), ...extra });

describe('the Monitor’s numbers', () => {
	it('says requests, errors with their share, p95 and visitors', () => {
		expect(monitorStats(metrics({ totals: counters({ requests: 214, errors_5xx: 3, p95_ms: 182.4, visitors: 37 }) }))).toEqual({ requests: '214', errors: '3', errorShare: '1.4%', p95: '182 ms', visitors: '37' });
	});

	it('leaves what wasn’t measured unsaid, never 0', () => {
		expect(monitorStats(metrics())).toEqual({ requests: null, errors: null, errorShare: null, p95: null, visitors: null });
		expect(monitorStats(metrics({ totals: counters({ requests: 0, errors_5xx: 0 }) })).errorShare).toBeNull();
	});
});

describe('the traffic bars', () => {
	it('scales each bucket to the busiest, keeps unmeasured ones apart, and finds the peak', () => {
		const buckets = [counters({ at: '2026-11-14T08:00:00Z', requests: 6 }), counters({ at: '2026-11-14T09:00:00Z', requests: 126 }), counters({ at: '2026-11-14T10:00:00Z', requests: null }), counters({ at: '2026-11-14T11:00:00Z', requests: 0 })] as MetricsBucket[];
		expect(trafficBars(buckets, 'UTC')).toEqual({
			bars: [
				{ at: '08:00', share: 6 / 126, count: 6 },
				{ at: '09:00', share: 1, count: 126 },
				{ at: '10:00', share: null, count: null },
				{ at: '11:00', share: 0, count: 0 },
			],
			peak: { at: '09:00', count: 126 },
		});
		expect(trafficBars([], 'UTC')).toEqual({ bars: [], peak: null });
	});

	it('counts visits instead, for a site', () => {
		const buckets = [counters({ at: '2026-11-14T08:00:00Z', requests: 90, visitors: 12 }), counters({ at: '2026-11-14T09:00:00Z', requests: 40, visitors: 48 })] as MetricsBucket[];
		expect(trafficBars(buckets, 'UTC', 'visitors')).toEqual({ bars: [{ at: '08:00', share: 12 / 48, count: 12 }, { at: '09:00', share: 1, count: 48 }], peak: { at: '09:00', count: 48 } });
	});
});

describe('the resources', () => {
	it('adds the app’s containers up: memory against its limit, CPU, bandwidth', () => {
		const usage: ContainerUsage[] = [
			{ service: 'web', container_id: 'a', stats: { cpu_percent: 3, memory_usage_bytes: 61 * 1024 * 1024, memory_limit_bytes: 256 * 1024 * 1024, memory_percent: 24, net_rx_bytes: 12 * 1024 * 1024, net_tx_bytes: 6 * 1024 * 1024 } },
			{ service: 'db', container_id: 'b', stats: null },
		];
		expect(resourceRows(usage)).toEqual({ memory: '61 / 256 MB', cpu: '3%', bandwidth: '18 MB' });
		expect(resourceRows([])).toEqual({ memory: null, cpu: null, bandwidth: null });
	});
});
