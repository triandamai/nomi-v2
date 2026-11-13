import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import CrewMemberRow from './CrewMemberRow.svelte';
import CrewMemberHeader from './CrewMemberHeader.svelte';
import CrewDetailRow from './CrewDetailRow.svelte';
import PersonalityVersions from './PersonalityVersions.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Crew's pieces (App / Settings / Crew vVUeh), measured against the pen.

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();
const varColor = (name: string) => {
	const probe = document.createElement('span');
	probe.style.color = `var(--${name})`;
	document.body.append(probe);
	const color = getComputedStyle(probe).color;
	probe.remove();
	return color;
};

describe('CrewMemberRow', () => {
	it('is a 42px link: the mascot 8 in, name and what it does, its state on the right', async () => {
		await page.viewport(1512, 862);
		await render(CrewMemberRow, { agent: 'kala', name: 'Kala', role: 'Reminders', state: 'on', href: '?member=reminders' });
		const row = page.getByRole('link', { name: /Kala/ }).element();
		expect([box(row).height, row.getAttribute('href')]).toEqual([42, '?member=reminders']);
		const mascot = row.querySelector('svg')!;
		expect([box(mascot).left - box(row).left, box(mascot).width]).toEqual([8, 20]);
		expect(getComputedStyle(page.getByText('Reminders').element()).color).toBe(tokenColor('nm-text-3'));
		expect(getComputedStyle(page.getByText('on', { exact: true }).element()).fontFamily).toMatch(/Mono/);
	});

	it('wears the member’s own tint when chosen', async () => {
		await render(CrewMemberRow, { agent: 'dana', name: 'Dana', role: 'Money records', state: 'on', href: '?member=money', selected: true });
		const row = page.getByRole('link', { name: /Dana/ }).element();
		expect(row.getAttribute('aria-current')).toBe('page');
		expect(getComputedStyle(row).backgroundColor).toBe(varColor('crew-dana-bg'));
		expect(getComputedStyle(page.getByText('Money records').element()).color).toBe(varColor('crew-dana-ink'));
		expect(getComputedStyle(page.getByText('Dana').element()).fontWeight).toBe('500');
	});

	it('marks a custom member with its initial, and a state that needs you in the warning colour', async () => {
		await render(CrewMemberRow, { initial: 'B', name: 'Bima', role: 'Investor updates', state: 'waiting', tone: 'warning', href: '?member=bima' });
		expect(page.getByText('B', { exact: true }).element().closest('[data-part="mark"]')).not.toBeNull();
		expect(getComputedStyle(page.getByText('waiting').element()).color).toBe(tokenColor('nm-warning'));
	});
});

describe('CrewMemberHeader', () => {
	it('is the member’s tinted header: 20 × 24 in, the 32px mascot, name, tag and detail', async () => {
		await page.viewport(1512, 862);
		await render(CrewMemberHeader, { agent: 'dana', name: 'Dana', tag: 'Specialist', detail: 'Records income and expenses' });
		const header = page.getByRole('banner').element();
		expect(getComputedStyle(header).backgroundColor).toBe(varColor('crew-dana-bg'));
		const mascot = header.querySelector('svg')!;
		expect([box(mascot).width, box(mascot).left - box(header).left]).toEqual([32, 24]);
		await expect.element(page.getByRole('heading', { name: 'Dana' })).toBeVisible();
		await expect.element(page.getByText('Specialist')).toBeVisible();
		expect(getComputedStyle(page.getByText('Records income and expenses').element()).fontFamily).toMatch(/Mono/);
	});
});

describe('CrewDetailRow', () => {
	it('is 48 high over its bottom line, with its control on the right', async () => {
		await render(CrewDetailRow, { title: 'Steps in', description: 'Money in any chat' });
		const row = page.getByText('Steps in').element().closest('[data-part="row"]')!;
		expect([box(row).height, getComputedStyle(row).borderBottomColor]).toEqual([48, tokenColor('nm-border')]);
		expect(getComputedStyle(page.getByText('Money in any chat').element()).color).toBe(tokenColor('nm-text-3'));
	});
});

describe('PersonalityVersions', () => {
	const versions = [
		{ version: 3, description: 'Warm and brief', date: '12 Oct', current: true },
		{ version: 2, description: 'Formal', date: '2 Oct', current: false },
	];

	it('lists the versions under the soul head, the current one tagged and the others restorable', async () => {
		const onrestore = vi.fn();
		await render(PersonalityVersions, { title: 'personality', count: 'v3 · 2 versions', versions, labels: { current: 'current', restore: 'Restore' }, onrestore });
		const head = page.getByText('personality', { exact: true }).element().parentElement!;
		expect([getComputedStyle(head).backgroundColor, box(head).height]).toEqual([tokenBackground('nm-surface'), 46]);
		await expect.element(page.getByText('current')).toBeVisible();
		expect(page.getByRole('button', { name: /Restore/ }).elements()).toHaveLength(1);
		await page.getByRole('button', { name: /Restore/ }).click();
		expect(onrestore).toHaveBeenCalledWith(2);
	});

	it('says so when there are no versions yet', async () => {
		await render(PersonalityVersions, { title: 'personality', count: 'no versions yet', versions: [], empty: 'Nomi keeps a version each time its personality changes.', labels: { current: 'current', restore: 'Restore' } });
		await expect.element(page.getByText('Nomi keeps a version each time its personality changes.')).toBeVisible();
		expect(page.getByRole('listitem').elements()).toHaveLength(0);
	});
});
