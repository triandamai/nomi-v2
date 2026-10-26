import { describe, expect, it } from 'vitest';
import { chartPaths, eventTone, filterLogs, liveSince, logLevel, orderContainers, peak } from './monitorView';
import type { ContainerUsage, ShipyardService } from './types';

describe('logLevel', () => {
	it('reads errors and warnings as whole words, in any case', () => {
		expect(logLevel('ERROR Payment provider timed out')).toBe('error');
		expect(logLevel('Uncaught exception in handler')).toBe('error');
		expect(logLevel('fatal: out of memory')).toBe('error');
		expect(logLevel('[warn] slow query (612ms)')).toBe('warn');
		expect(logLevel('Warning: deprecated option')).toBe('warn');
		expect(logLevel('GET /errorless 200')).toBe('info');
		expect(logLevel('GET /menu 200 18ms')).toBe('info');
	});
});

describe('filterLogs', () => {
	const lines = ['GET /menu 200', 'warn slow query', 'error Payment provider timed out', 'POST /api/orders 201'];
	it('keeps everything, warnings and up, or errors only', () => {
		expect(filterLogs(lines, 'all', '')).toEqual(lines);
		expect(filterLogs(lines, 'warn', '')).toEqual(['warn slow query', 'error Payment provider timed out']);
		expect(filterLogs(lines, 'error', '')).toEqual(['error Payment provider timed out']);
	});
	it('searches without caring about case', () => {
		expect(filterLogs(lines, 'all', 'ORDERS')).toEqual(['POST /api/orders 201']);
	});
});

describe('peak and chart', () => {
	it('finds the peak, or nothing without samples', () => {
		expect(peak([12, 41, 30])).toBe(41);
		expect(peak([])).toBeNull();
	});
	it('needs two samples to draw, and scales to at least 100%', () => {
		expect(chartPaths([10], 100, 40)).toBeNull();
		const paths = chartPaths([0, 100], 100, 40)!;
		expect(paths.line).toBe('M0,40 L100,0');
		expect(paths.area).toBe('M0,40 L100,0 L100,40 L0,40 Z');
		expect(chartPaths([0, 50], 100, 40)!.line).toBe('M0,40 L100,20');
	});
});

describe('events and services', () => {
	it('gives each event a tone', () => {
		expect(eventTone('service.crashed')).toBe('failed');
		expect(eventTone('publish.failed')).toBe('failed');
		expect(eventTone('service.resource_alert')).toBe('running');
		expect(eventTone('publish.succeeded')).toBe('succeeded');
	});
	const service = (name: string, created_at: string): ShipyardService => ({
		name, service_id: name, type: name === 'db' ? 'database' : 'web', status: 'running', replicas: 1, running: 1, urls: [],
		last_deployment: { id: 'd', status: 'succeeded', created_at },
	});
	it('says the app is up since the web service last deployed', () => {
		expect(liveSince([service('db', '2026-10-20T00:00:00Z'), service('web', '2026-10-24T09:00:00Z')])).toBe('2026-10-24T09:00:00Z');
		expect(liveSince([])).toBeNull();
	});
	it('lists the database container last', () => {
		const c = (service: string) => ({ service, container_id: service }) as ContainerUsage;
		expect(orderContainers([c('db'), c('web'), c('worker')]).map((x) => x.service)).toEqual(['web', 'worker', 'db']);
	});
});
