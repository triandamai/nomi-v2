import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import ActionSheet from '../overlays/ActionSheet.svelte';
import DeploySheet from './DeploySheet.svelte';
import DeleteProjectSheet from './DeleteProjectSheet.svelte';
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
