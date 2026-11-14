import { describe, expect, it } from 'vitest';
import { MB_PER_GB, formatStorage, parseLimit, readPlanLimits } from './planLimits';

function form(entries: [string, string][]) {
	const data = new FormData();
	for (const [key, value] of entries) data.append(key, value);
	return data;
}

describe('parseLimit', () => {
	it('reads an empty input as unlimited, not zero', () => {
		expect(parseLimit('')).toBeNull();
		expect(parseLimit('  ')).toBeNull();
		expect(parseLimit(null)).toBeNull();
	});

	it('reads whole numbers, zero included', () => {
		expect(parseLimit('3')).toBe(3);
		expect(parseLimit(' 20 ')).toBe(20);
		expect(parseLimit('0')).toBe(0);
	});

	it('treats anything else as unlimited', () => {
		expect(parseLimit('-1')).toBeNull();
		expect(parseLimit('1.5')).toBeNull();
		expect(parseLimit('lots')).toBeNull();
	});
});

describe('readPlanLimits', () => {
	it('defaults to unlimited, nothing extra and no model list', () => {
		expect(readPlanLimits(form([]))).toEqual({
			max_projects: null,
			max_orgs: null,
			max_org_members: null,
			max_custom_crew: null,
			max_storage_mb: null,
			custom_domain: false,
			sandbox_size: null,
			all_models: false,
			model_ids: [],
		});
	});

	it('reads numbers, switches, the sandbox size and the model list', () => {
		const limits = readPlanLimits(
			form([
				['max_projects', '3'],
				['max_orgs', ''],
				['max_org_members', '5'],
				['max_custom_crew', '0'],
				['max_storage_mb', '1024'],
				['custom_domain', 'true'],
				['sandbox_size', 'large'],
				['all_models', 'false'],
				['model_ids', 'a'],
				['model_ids', 'b'],
			]),
		);
		expect(limits).toEqual({
			max_projects: 3,
			max_orgs: null,
			max_org_members: 5,
			max_custom_crew: 0,
			max_storage_mb: 1024,
			custom_domain: true,
			sandbox_size: 'large',
			all_models: false,
			model_ids: ['a', 'b'],
		});
	});

	it('reads the platform default sandbox (empty or unknown) as null', () => {
		expect(readPlanLimits(form([['sandbox_size', '']])).sandbox_size).toBeNull();
		expect(readPlanLimits(form([['sandbox_size', 'huge']])).sandbox_size).toBeNull();
		expect(readPlanLimits(form([['sandbox_size', 'default']])).sandbox_size).toBe('default');
	});

	it('reads all models on', () => {
		expect(readPlanLimits(form([['all_models', 'true']])).all_models).toBe(true);
	});
});

describe('formatStorage', () => {
	it('shows MB below a gigabyte and GB from there', () => {
		expect(formatStorage(512)).toBe('512 MB');
		expect(formatStorage(MB_PER_GB)).toBe('1 GB');
		expect(formatStorage(MB_PER_GB * 1.5)).toBe('1.5 GB');
		expect(formatStorage(MB_PER_GB * 10)).toBe('10 GB');
	});
});
