import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import ActionSheet from '../overlays/ActionSheet.svelte';
import DeploySheet from './DeploySheet.svelte';
import DeleteProjectSheet from './DeleteProjectSheet.svelte';
import ProjectMoveSheet from './ProjectMoveSheet.svelte';
import ProjectCardLoading from './ProjectCardLoading.svelte';
import LoadError from './LoadError.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Flow / Mobile Projects' confirmations on a phone: the action sheet that stops a build, the sheet
// that deploys a draft, and the one that deletes a project once its name is typed.

afterEach(() => {
	document.body.innerHTML = '';
});
const box = (element: Element) => element.getBoundingClientRect();

describe('ActionSheet', () => {
	it('asks over two rounded groups: the question, its actions at 18 (the destructive one in danger), and the way out', async () => {
		await page.viewport(390, 844);
		const onstop = vi.fn();
		const onexport = vi.fn();
		await render(ActionSheet, {
			open: true,
			title: 'Stop building warung-kopi-menu?',
			body: 'Koda finishes the file it’s writing, then stops.',
			actions: [
				{ label: 'Export first', onclick: onexport },
				{ label: 'Stop building', danger: true, onclick: onstop },
			],
			cancel: 'Keep building',
		});
		const stop = page.getByRole('button', { name: 'Stop building' }).element();
		expect([box(stop).height, getComputedStyle(stop).fontSize, getComputedStyle(stop).color]).toEqual([56, '18px', tokenColor('nm-danger')]);
		const keep = page.getByRole('button', { name: 'Keep building' }).element();
		expect([getComputedStyle(keep).fontWeight, getComputedStyle(keep).borderRadius]).toEqual(['600', '16px']);
		await page.getByRole('button', { name: 'Stop building' }).click();
		expect(onstop).toHaveBeenCalledOnce();
		await expect.element(page.getByRole('dialog')).not.toBeInTheDocument();
	});
});

describe('DeploySheet', () => {
	it('says what deploying does, its facts, and deploys from a 46 accent button', async () => {
		await page.viewport(390, 844);
		const ondeploy = vi.fn();
		await render(DeploySheet, { open: true, name: 'budget-keluarga', files: 3, url: 'a new address under trian.space', data: 'none yet', ondeploy });
		await expect.element(page.getByRole('dialog', { name: 'Deploy budget-keluarga?' })).toBeVisible();
		const deploy = page.getByRole('button', { name: 'Deploy' }).element();
		expect([box(deploy).height, getComputedStyle(deploy).backgroundColor]).toEqual([46, tokenBackground('nm-accent')]);
		await page.getByRole('button', { name: 'Deploy' }).click();
		expect(ondeploy).toHaveBeenCalledOnce();
	});
});

describe('DeleteProjectSheet', () => {
	it('deletes only once the name is typed as it is, in the 48 danger field', async () => {
		await page.viewport(390, 844);
		const ondelete = vi.fn();
		await render(DeleteProjectSheet, { open: true, name: 'salah-tracker', ondelete, onexport: () => {} });
		const field = page.getByRole('textbox', { name: 'Type salah-tracker to confirm' });
		expect([box(field.element()).height, getComputedStyle(field.element()).borderTopColor]).toEqual([48, tokenColor('nm-danger')]);
		const remove = page.getByRole('button', { name: 'Delete project' });
		await expect.element(remove).toBeDisabled();
		await field.fill('salah');
		await expect.element(remove).toBeDisabled();
		await field.fill('salah-tracker');
		await userEvent.keyboard('{Enter}');
		expect(ondelete).toHaveBeenCalledOnce();
		await expect.element(page.getByRole('button', { name: 'Export repo first' })).toBeVisible();
	});
});

describe('ProjectMoveSheet', () => {
	const workspaces = [
		{ id: 'o1', name: 'Mario', meta: 'personal · only you', tone: 'plain' as const, members: 1 },
		{ id: 'o2', name: "Trian's space", meta: '3 members', tone: 'plain' as const, members: 3, current: true },
		{ id: 'o3', name: 'Kopi Kita', meta: 'Admin · 5 members', tone: 'plain' as const, members: 5, owner: 'Andi' },
	];
	it('moves to the workspace chosen among the cards, the current one not choosable', async () => {
		await page.viewport(390, 844);
		const onmove = vi.fn();
		await render(ProjectMoveSheet, { open: true, name: 'salah-tracker', chats: 2, workspaces, onmove });
		const move = page.getByRole('button', { name: 'Move project' });
		await expect.element(move).toBeDisabled();
		await expect.element(page.getByRole('radio', { name: /Trian/ })).toHaveAttribute('aria-disabled', 'true');
		await page.getByRole('radio', { name: /Kopi Kita/ }).click();
		await expect.element(page.getByRole('radio', { name: /Kopi Kita/ })).toHaveAttribute('aria-checked', 'true');
		expect(box(move.element()).height).toBe(46);
		await move.click();
		expect(onmove).toHaveBeenCalledWith('o3');
	});
});

describe('Loading and couldn’t load, on a phone', () => {
	it('draws phone loading cards, and the centred error with Retry as a pill', async () => {
		await page.viewport(390, 844);
		await render(ProjectCardLoading, { phone: true });
		const card = page.getByRole('status', { name: 'Loading a project' }).element();
		expect([Math.round(box(card).width), getComputedStyle(card).borderRadius]).toEqual([390, '14px']);
		document.body.innerHTML = '';
		const onretry = vi.fn();
		await render(LoadError, { phone: true, reason: '503 from api.nomi.app · 09:14', onretry });
		await expect.element(page.getByText('Couldn’t load your projects')).toBeVisible();
		const retry = page.getByRole('button', { name: 'Retry' });
		expect(box(retry.element()).height).toBe(40);
		await retry.click();
		expect(onretry).toHaveBeenCalledOnce();
	});
});
