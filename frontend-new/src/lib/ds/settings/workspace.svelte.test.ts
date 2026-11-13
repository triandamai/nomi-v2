import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { Folder, Mail, Users } from '@lucide/svelte';
import WorkspaceStats from './WorkspaceStats.svelte';
import InviteBox from './InviteBox.svelte';
import PersonRow from './PersonRow.svelte';
import WorkspaceBoundary from './WorkspaceBoundary.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Workspace's pieces (App / Settings / Workspace wZm9B), measured against the pen.

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();
const labels = { field: 'Email addresses', placeholder: 'Add more, separated by commas', remove: 'Remove {email}', role: 'Role', note: 'they won’t see your chats, memory or money', send: (count: number) => `Send ${count} invites` };
const roles = [
	{ value: 'member', title: 'Member', body: 'Works on projects and their chats' },
	{ value: 'admin', title: 'Admin', body: 'Also invites, removes people and renames' },
];

describe('WorkspaceStats', () => {
	it('is a 64px outlined strip, each stat [14,16] with its icon, number in mono and label', async () => {
		await page.viewport(1512, 862);
		await render(WorkspaceStats, { label: 'In this workspace', stats: [{ icon: Folder, value: 4, label: 'Projects' }, { icon: Users, value: 3, label: 'Members' }, { icon: Mail, value: 1, label: 'Pending invites' }] });
		const strip = page.getByRole('list', { name: 'In this workspace' }).element();
		expect([box(strip).height, getComputedStyle(strip).outlineColor]).toEqual([64, tokenColor('nm-border')]);
		const cells = strip.querySelectorAll('li');
		expect(getComputedStyle(cells[1]).borderLeftColor).toBe(tokenColor('nm-border'));
		expect(getComputedStyle(page.getByText('4', { exact: true }).element()).fontFamily).toMatch(/Mono/);
	});
});

describe('InviteBox', () => {
	it('turns addresses into chips on a comma or Enter, and takes one back', async () => {
		const onsend = vi.fn();
		await render(InviteBox, { labels, roles, role: 'member', onsend });
		const field = page.getByRole('textbox', { name: 'Email addresses' });
		await field.fill('siti@kopi.id,');
		await field.fill('budi@gmail.com');
		await userEvent.keyboard('{Enter}');
		await expect.element(page.getByText('siti@kopi.id')).toBeVisible();
		await expect.element(page.getByText('budi@gmail.com')).toBeVisible();
		await field.fill('nope,');
		expect(getComputedStyle(page.getByText('nope').element()).color).toBe(tokenColor('nm-danger'));
		await page.getByRole('button', { name: 'Remove nope' }).click();
		await page.getByRole('button', { name: 'Remove siti@kopi.id' }).click();
		expect(page.getByText('siti@kopi.id').elements()).toHaveLength(0);
		await page.getByRole('button', { name: 'Send 1 invites' }).click();
		expect(onsend).toHaveBeenCalledWith(['budi@gmail.com'], 'member');
	});

	it('sends what’s still typed too, as the role chosen', async () => {
		const onsend = vi.fn();
		await render(InviteBox, { labels, roles, role: 'member', onsend });
		await page.getByRole('textbox', { name: 'Email addresses' }).fill('andi@kopikita.id');
		await page.getByText('Admin', { exact: true }).click();
		await expect.element(page.getByRole('radio', { name: /Admin/ })).toBeChecked();
		await page.getByRole('button', { name: /^Send/ }).click();
		expect(onsend).toHaveBeenCalledWith(['andi@kopikita.id'], 'admin');
	});

	it('is the pen’s box: the surface, 14 in, the role chosen on surface-2 and the error under it', async () => {
		await page.viewport(1512, 862);
		await render(InviteBox, { labels, roles, role: 'member', error: 'Check these addresses: nope' });
		const boxEl = page.getByRole('radiogroup', { name: 'Role' }).element().parentElement!;
		expect([getComputedStyle(boxEl).backgroundColor, getComputedStyle(boxEl).paddingTop]).toEqual([tokenBackground('nm-surface'), '14px']);
		const chosen = page.getByRole('radio', { name: /Member/ }).element().closest('label')!;
		expect([box(chosen).height, getComputedStyle(chosen).backgroundColor]).toEqual([54, tokenBackground('nm-surface-2')]);
		await expect.element(page.getByText('Check these addresses: nope')).toBeVisible();
	});
});

describe('PersonRow', () => {
	it('is the pen’s member row: 56 over its line with “you” (55 without), the 32px face, the meta in mono', async () => {
		await page.viewport(1512, 862);
		await render(PersonRow, { name: 'Mario Prasetyo', initial: 'M', you: 'you', meta: 'ko••@gmail.com · joined 14 Jul' });
		const row = page.getByText('Mario Prasetyo').element().closest('li')!;
		expect([box(row).height, getComputedStyle(row).borderBottomColor]).toEqual([56, tokenColor('nm-border')]);
		const face = row.querySelector('[data-part="face"]')!;
		expect(box(face).width).toBe(32);
		await expect.element(page.getByText('you', { exact: true })).toBeVisible();
		expect(getComputedStyle(page.getByText('ko••@gmail.com · joined 14 Jul').element()).fontFamily).toMatch(/Mono/);
	});
});

describe('WorkspaceBoundary', () => {
	it('lists what’s shared and what stays personal, with the tip under a line', async () => {
		await render(WorkspaceBoundary, { shared: { title: 'Shared here', items: ['Projects and their chats'] }, personal: { title: 'Always personal', items: ['Memory and profile'] }, tip: 'Switch workspaces from the sidebar.' });
		await expect.element(page.getByText('Shared here')).toBeVisible();
		await expect.element(page.getByText('Memory and profile')).toBeVisible();
		const tip = page.getByText('Switch workspaces from the sidebar.').element().parentElement!;
		expect(getComputedStyle(tip).borderTopColor).toBe(tokenColor('nm-border'));
	});
});
