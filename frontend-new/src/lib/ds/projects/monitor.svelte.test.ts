import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { Cpu } from '@lucide/svelte';
import MonitorStat from './MonitorStat.svelte';
import TrafficBars from './TrafficBars.svelte';
import ResourceRow from './ResourceRow.svelte';
import LogLines from './LogLines.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Monitor (Flow / Build — Workspace tools U16wU, Screen / Monitor EQNtT), measured against the pen.

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();

describe('MonitorStat', () => {
	it('is the pen’s 79px tile: [12,14], outlined, its label, value in mono and note', async () => {
		await page.viewport(1512, 862);
		await render(MonitorStat, { label: 'Requests', value: '214', note: 'today' });
		const tile = page.getByText('214').element().closest('[data-part="stat"]')!;
		expect([box(tile).height, getComputedStyle(tile).outlineColor]).toEqual([79, tokenColor('nm-border')]);
		expect(getComputedStyle(page.getByText('214').element()).fontFamily).toMatch(/Mono/);
	});

	it('says when something wasn’t measured', async () => {
		await render(MonitorStat, { label: 'P95', notMeasured: 'Not measured' });
		expect(getComputedStyle(page.getByText('Not measured').element()).color).toBe(tokenColor('nm-text-3'));
	});
});

describe('TrafficBars', () => {
	it('draws each bucket in 96, the busiest full, quiet ones as a line, gaps as nothing', async () => {
		await page.viewport(1512, 862);
		await render(TrafficBars, { label: 'Requests per hour', bars: [{ at: '08:00', share: 1, count: 120 }, { at: '09:00', share: 0, count: 0 }, { at: '10:00', share: null, count: null }], axis: ['00', '12', '24'] });
		const chart = page.getByRole('img', { name: 'Requests per hour' }).element();
		const bars = [...chart.querySelectorAll('[data-part="bar"]')];
		expect(box(chart.querySelector('[data-part="bars"]')!).height).toBe(96);
		expect([box(bars[0]).height, getComputedStyle(bars[0]).backgroundColor]).toEqual([96, tokenBackground('nm-text-2')]);
		expect([box(bars[1]).height, getComputedStyle(bars[1]).backgroundColor]).toEqual([2, tokenBackground('nm-border')]);
		expect(box(bars[2]).height).toBe(0);
	});
});

describe('ResourceRow and LogLines', () => {
	it('lists a resource with its icon, name (72 wide) and value in mono', async () => {
		await render(ResourceRow, { icon: Cpu, name: 'CPU', value: '4%' });
		expect(box(page.getByText('CPU').element()).width).toBe(72);
		expect(getComputedStyle(page.getByText('4%').element()).fontFamily).toMatch(/Mono/);
	});

	it('prints the app’s lines in mono, errors in the danger colour', async () => {
		await render(LogLines, { label: 'Live log', lines: ['GET /menu 200', 'ERROR db timeout'] });
		const log = page.getByRole('log', { name: 'Live log' }).element();
		expect(getComputedStyle(log).fontFamily).toMatch(/Mono/);
		expect(getComputedStyle(page.getByText('ERROR db timeout').element()).color).toBe(tokenColor('nm-danger'));
	});
});
