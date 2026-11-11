import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import StopDialog from './StopDialog.svelte';
import DeployDialog from './DeployDialog.svelte';
import ProjectMoveDialog from './ProjectMoveDialog.svelte';
import InviteDialog from './InviteDialog.svelte';
import ProjectDialogPanel from './ProjectDialogPanel.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Design System / Projects → Cards & overlays: the confirms (m7QPG7, A3CDJ) and the workspace
// dialogs (dEQNS, RtsT2), drawn as one dialog panel (surface, border-strong inside, the float
// shadow) with a footer 54 tall.

const style = (element: Element) => getComputedStyle(element);
const box = (element: Element) => element.getBoundingClientRect();

// Wide enough for the grid's two columns and the widest dialog, as on a desktop.
beforeEach(async () => {
	await page.viewport(1280, 800);
});
const panel = () => page.getByTestId('dialog-panel').element();

describe('ProjectDialogPanel', () => {
	it('takes the pen’s four widths', async () => {
		for (const width of [420, 440, 460, 520] as const) {
			const { unmount } = await render(ProjectDialogPanel, { id: 'd', title: 'Title', width });
			expect(box(panel()).width).toBe(width);
			unmount();
		}
	});
});

describe('StopDialog (m7QPG7)', () => {
	it('is 420 × 181 and says what stopping keeps', async () => {
		const onstop = vi.fn();
		await render(StopDialog, { inline: true, name: 'warung-kopi-menu', done: 3, next: 4, onstop });
		expect(box(panel()).width).toBe(420);
		expect(box(panel()).height).toBe(181);
		expect(style(panel()).outlineColor).toBe(tokenColor('nm-border-strong'));
		await expect.element(page.getByRole('heading', { name: 'Stop building warung-kopi-menu?' })).toBeInTheDocument();
		await expect.element(page.getByText(/The 3 steps done so far stay in the project/)).toBeInTheDocument();
		await expect.element(page.getByText(/Resume picks up at step 4\./)).toBeInTheDocument();
		const footer = page.getByRole('button', { name: 'Keep building' }).element().parentElement!;
		expect(box(footer).height).toBe(54);
		await page.getByRole('button', { name: 'Stop' }).click();
		expect(onstop).toHaveBeenCalledOnce();
	});

	it('opens as a modal and Keep building closes it', async () => {
		await render(StopDialog, { open: true, name: 'warung-kopi-menu', done: 3, next: 4, onstop: () => {} });
		await expect.element(page.getByRole('dialog', { name: 'Stop building warung-kopi-menu?' })).toBeVisible();
		await page.getByRole('button', { name: 'Keep building' }).click();
		await expect.element(page.getByRole('dialog')).not.toBeInTheDocument();
	});
});

describe('DeployDialog (A3CDJ)', () => {
	it('is 420 × 249 with the facts in a 80 tall box: key 56, value in mono', async () => {
		const ondeploy = vi.fn();
		await render(DeployDialog, {
			inline: true,
			name: 'budget-keluarga',
			files: 3,
			url: 'budget-keluarga-7c21.trian.space',
			data: 'none yet · add a database later',
			uses: '1 build · 26 of 30 left in Oct',
			ondeploy,
		});
		expect(box(panel()).height).toBe(249);
		const facts = page.getByRole('definition').elements();
		expect(facts).toHaveLength(3);
		expect(box(facts[0].parentElement!.parentElement!).height).toBe(80);
		expect(style(facts[0]).fontFamily).toContain('Geist Mono');
		expect(box(page.getByRole('term').elements()[0]).width).toBe(56);
		await page.getByRole('button', { name: 'Deploy' }).click();
		expect(ondeploy).toHaveBeenCalledOnce();
	});
});

describe('ProjectMoveDialog (dEQNS)', () => {
	const workspaces = [
		{ id: 'w1', name: 'Mario', meta: 'personal · only you', tone: 'plain' as const, members: 1 },
		{ id: 'w2', name: "Trian's space", meta: 'current · 3 members', tone: 'nomi' as const, members: 3, current: true },
		{ id: 'w3', name: 'Kopi Kita', meta: 'Admin · 5 members', tone: 'tara' as const, members: 5 },
	];

	it('is 440 × 359: the workspaces as a radio group, 44 rows, what changes for whom', async () => {
		await render(ProjectMoveDialog, { inline: true, name: 'salah-daily-tracker', chats: 2, owner: 'Trian', workspaces, value: 'w3', onmove: () => {} });
		expect(box(panel()).width).toBe(440);
		expect(box(panel()).height).toBe(359);
		const radios = page.getByRole('radio').elements();
		expect(box(radios[0]).height).toBe(44);
		await expect.element(page.getByRole('radio', { name: /Kopi Kita/ })).toHaveAttribute('aria-checked', 'true');
		expect(style(radios[2]).backgroundColor).toBe(tokenBackground('nm-surface-2'));
		await expect.element(page.getByRole('radio', { name: /Trian's space/ })).toHaveAttribute('aria-disabled', 'true');
		await expect.element(page.getByText("The 5 members of Kopi Kita will see it and its 2 chats. Trian's space loses access.")).toBeInTheDocument();
		await expect.element(page.getByText(/Trian owns this workspace, so Trian approves the move\./)).toBeInTheDocument();
	});

	it('moves to the chosen workspace; arrows choose', async () => {
		const onmove = vi.fn();
		await render(ProjectMoveDialog, { inline: true, name: 'salah-daily-tracker', chats: 2, owner: 'Trian', workspaces, value: 'w3', onmove });
		page.getByRole('radio', { name: /Kopi Kita/ }).element().focus();
		await userEvent.keyboard('{ArrowDown}');
		await expect.element(page.getByRole('radio', { name: /Mario/ })).toHaveAttribute('aria-checked', 'true');
		await page.getByRole('button', { name: 'Move project' }).click();
		expect(onmove).toHaveBeenCalledWith('w1');
	});
});

describe('InviteDialog (RtsT2)', () => {
	it('is 520 × 277: addresses as chips, two roles side by side', async () => {
		await render(InviteDialog, { inline: true, workspace: "Trian's space", emails: ['siti@kopikita.id', 'budi.s@gmail.com'], oninvite: () => {} });
		expect(box(panel()).width).toBe(520);
		expect(box(panel()).height).toBe(277);
		const field = page.getByRole('textbox', { name: 'Email addresses' }).element().parentElement!;
		expect(box(field).height).toBe(38);
		expect(style(field).borderTopColor).toBe(tokenColor('nm-accent'));
		const member = page.getByRole('radio', { name: /Member/ }).element();
		expect(box(member).height).toBe(58);
		expect(member.getAttribute('aria-checked')).toBe('true');
		expect(style(member).borderTopColor).toBe(tokenColor('nm-text-2'));
		await expect.element(page.getByRole('button', { name: 'Send 2 invites' })).toBeInTheDocument();
	});

	it('adds an address on Enter or comma, removes one, and sends with the role', async () => {
		const oninvite = vi.fn();
		await render(InviteDialog, { inline: true, workspace: "Trian's space", oninvite });
		await expect.element(page.getByRole('button', { name: 'Send invite' })).toBeDisabled();
		const field = page.getByRole('textbox', { name: 'Email addresses' });
		await field.fill('siti@kopikita.id');
		await userEvent.keyboard('{Enter}');
		await field.fill('budi.s@gmail.com,');
		await expect.element(page.getByRole('button', { name: 'Remove budi.s@gmail.com' })).toBeInTheDocument();
		await page.getByRole('button', { name: 'Remove siti@kopikita.id' }).click();
		await page.getByRole('radio', { name: /Admin/ }).click();
		await page.getByRole('button', { name: 'Send invite' }).click();
		expect(oninvite).toHaveBeenCalledWith(['budi.s@gmail.com'], 'admin');
	});
});
