import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import ProjectCard from './ProjectCard.svelte';
import ProjectCardLoading from './ProjectCardLoading.svelte';
import ProjectCardMenu, { projectMenu } from './ProjectCardMenu.svelte';
import ProjectMenuPanel from './ProjectMenuPanel.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Design System / Projects → Cards & overlays (DLQ3G): the project card in each of its states
// (ig0xO, b0OUv, EU6Mi, F6qib, mwu3Z, qHlDT, m37KA, hUIVu, JyzIn, q7MGP) and its ⋯ menu (uA0lh).

const style = (element: Element) => getComputedStyle(element);
const box = (element: Element) => element.getBoundingClientRect();

// Wide enough for the grid's two columns and the widest dialog, as on a desktop.
beforeEach(async () => {
	await page.viewport(1280, 800);
});
const base = { name: 'salah-daily-tracker', description: 'Daily salah tracker with countdown and streaks', tone: 'nomi' as const, footer: '2 chats · deploy #15 · 2d ago' };
const zone = () => page.getByTestId('status-zone').element();

describe('ProjectCard', () => {
	it('is 384 × 192: identity, a 58 status zone and a divided footer, 12 apart', async () => {
		await render(ProjectCard, { ...base, status: 'live', url: 'salah-daily-tracker-0ab8f8.trian.space', href: '/projects/p1' });
		const card = page.getByRole('article', { name: 'salah-daily-tracker' }).element();
		expect(box(card).width).toBe(384);
		expect(box(card).height).toBe(192);
		expect(style(card).outlineColor).toBe(tokenColor('nm-border-strong'));
		expect(style(card).backgroundColor).toBe(tokenBackground('nm-surface'));
		expect(box(zone()).height).toBe(58);
		expect(box(zone()).top - box(card).top).toBe(64);
		expect(style(zone()).backgroundColor).toBe(tokenBackground('nm-bg'));
		const tile = page.getByText('S', { exact: true }).element().parentElement!;
		expect(box(tile).width).toBe(36);
		expect(style(page.getByText('S', { exact: true }).element()).fontFamily).toContain('Fraunces');
		const footer = page.getByText('2 chats · deploy #15 · 2d ago').element().parentElement!;
		expect(box(footer).height).toBe(42);
		expect(style(footer).borderTopColor).toBe(tokenColor('nm-border'));
	});

	it('when live: the live dot, the URL to visit, and Open', async () => {
		await render(ProjectCard, { ...base, status: 'live', url: 'salah-daily-tracker-0ab8f8.trian.space', href: '/projects/p1' });
		await expect.element(page.getByText('Live', { exact: true })).toBeInTheDocument();
		await expect.element(page.getByRole('link', { name: /salah-daily-tracker-0ab8f8\.trian\.space/ })).toHaveAttribute('href', 'https://salah-daily-tracker-0ab8f8.trian.space');
		await expect.element(page.getByRole('link', { name: 'Open' })).toHaveAttribute('href', '/projects/p1');
	});

	it('while Koda builds: its mascot, the step, the file, and a quiet Stop', async () => {
		const onaction = vi.fn();
		await render(ProjectCard, { ...base, status: 'building', meta: '3 of 6', detail: 'writing routes/menu/+page.svelte', footer: '1 chat · started 08:13', onaction });
		await expect.element(page.getByText('Koda is building')).toBeInTheDocument();
		expect(zone().querySelector('svg [data-part="body"]')).not.toBeNull();
		expect(style(page.getByText('3 of 6').element()).fontFamily).toContain('Geist Mono');
		await page.getByRole('button', { name: 'Stop' }).click();
		expect(onaction).toHaveBeenCalledOnce();
	});

	it('tints the zone: danger when something broke, warning when Koda needs you', async () => {
		const { rerender } = await render(ProjectCard, { ...base, status: 'failed', meta: 'deploy #6', detail: 'build error in src/lib/pdf.ts · #5 still live', onaction: () => {} });
		expect(style(zone()).backgroundColor).toBe(tokenBackground('nm-danger-bg'));
		expect(style(page.getByText('Deploy failed').element()).color).toBe(tokenColor('nm-danger'));
		expect(style(page.getByText('build error in src/lib/pdf.ts · #5 still live').element()).color).toBe(tokenColor('nm-text-2'));
		const fix = page.getByRole('button', { name: 'Ask Koda to fix' }).element();
		expect(style(fix).backgroundColor).toBe(tokenBackground('nm-text'));
		await rerender({ ...base, status: 'attention', questions: 3, detail: 'who uses it · how orders come in · prices', onaction: () => {} });
		expect(style(zone()).backgroundColor).toBe(tokenBackground('nm-warning-bg'));
		await expect.element(page.getByText('Koda has 3 questions')).toBeInTheDocument();
		await expect.element(page.getByRole('button', { name: 'Answer' })).toBeInTheDocument();
		await rerender({ ...base, status: 'down', meta: '4 min', detail: '502 for visitors · last OK 09:10', onaction: () => {} });
		expect(style(zone()).backgroundColor).toBe(tokenBackground('nm-danger-bg'));
		await expect.element(page.getByRole('button', { name: 'Ask Koda to check' })).toBeInTheDocument();
	});

	it('names each state’s one action', async () => {
		const cases = [
			['stopped', 'Stopped', 'Resume'],
			['draft', 'Draft', 'Deploy'],
			['importing', 'Koda is importing', 'Cancel'],
		] as const;
		for (const [status, label, action] of cases) {
			const { unmount } = await render(ProjectCard, { ...base, status, detail: 'detail', onaction: () => {} });
			await expect.element(page.getByText(label, { exact: true })).toBeInTheDocument();
			await expect.element(page.getByRole('button', { name: action })).toBeInTheDocument();
			unmount();
		}
	});

	it('renames in place: Enter saves, Escape cancels', async () => {
		const onrename = vi.fn();
		const oncancelrename = vi.fn();
		await render(ProjectCard, { ...base, status: 'live', url: 'salah-daily-tracker-0ab8f8.trian.space', href: '/p', renaming: true, onrename, oncancelrename });
		const field = page.getByRole('textbox', { name: 'Project name' });
		expect(box(field.element().parentElement!).height).toBe(20);
		expect(style(field.element().parentElement!).borderTopColor).toBe(tokenColor('nm-accent'));
		await expect.element(page.getByText('↵ save · esc cancel · URL stays the same')).toBeInTheDocument();
		await field.fill('salah-tracker');
		await userEvent.keyboard('{Enter}');
		expect(onrename).toHaveBeenCalledWith('salah-tracker');
		field.element().focus();
		await userEvent.keyboard('{Escape}');
		expect(oncancelrename).toHaveBeenCalledOnce();
	});

	it('shows being selected with a 2px stroke', async () => {
		await render(ProjectCard, { ...base, status: 'draft', detail: 'not deployed yet · 3 files', selected: true, onaction: () => {} });
		const card = page.getByRole('article').element();
		expect(style(card).outlineWidth).toBe('2px');
		expect(style(card).outlineColor).toBe(tokenColor('nm-text-2'));
	});

	it('opens its menu from ⋯ and takes F2 to rename', async () => {
		const onrename = vi.fn();
		await render(ProjectCard, { ...base, status: 'live', url: 'x.trian.space', href: '/p', actions: { onopen: () => {}, onrename } });
		await page.getByRole('button', { name: 'salah-daily-tracker actions' }).click();
		await expect.element(page.getByRole('menuitem', { name: /Open workspace/ })).toHaveFocus();
		await userEvent.keyboard('{Escape}');
		await expect.element(page.getByRole('menu')).not.toBeInTheDocument();
		await userEvent.keyboard('{F2}');
		expect(onrename).toHaveBeenCalledOnce();
	});
});

describe('ProjectCardLoading (q7MGP)', () => {
	it('stands in at the card’s size, quieter stroke, saying what loads', async () => {
		await render(ProjectCardLoading);
		const card = page.getByRole('status', { name: 'Loading a project' }).element();
		expect(box(card).width).toBe(384);
		expect(box(card).height).toBe(192);
		expect(style(card).outlineColor).toBe(tokenColor('nm-border'));
	});
});

describe('ProjectMenuPanel (uA0lh) and ProjectCardMenu', () => {
	it('is 248 wide: the name in mono capitals, rows 30, dividers between groups', async () => {
		const items = projectMenu(
			{ onopen: () => {}, onvisit: () => {}, onrename: () => {}, onduplicate: () => {}, onmove: () => {}, onshare: () => {}, onexport: () => {}, onsettings: () => {}, ontakedown: () => {}, ondelete: () => {} },
			true,
		);
		await render(ProjectMenuPanel, { title: 'salah-daily-tracker', items });
		const menu = page.getByRole('menu').element();
		expect(box(menu).width).toBe(248);
		expect(box(menu).height).toBe(361);
		expect(style(menu).boxShadow).not.toBe('none');
		const title = page.getByText('salah-daily-tracker').element();
		expect(style(title).textTransform).toBe('uppercase');
		expect(box(title).top - box(menu).top).toBe(4);
		expect(box(title).height).toBe(24);
		expect(style(title).paddingTop).toBe('6px');
		expect(page.getByRole('menuitem').elements()).toHaveLength(10);
		expect(menu.querySelectorAll('hr')).toHaveLength(3);
		await expect.element(page.getByRole('menuitem', { name: /Rename/ })).toHaveTextContent('F2');
		expect(style(page.getByRole('menuitem', { name: 'Delete project' }).element()).color).toBe(tokenColor('nm-danger'));
	});

	it('leaves out what the page can’t do, and Visit when nothing is live', async () => {
		expect(projectMenu({ onopen: () => {}, ondelete: () => {} }, false).filter((entry) => entry !== 'divider').map((entry) => (entry as { label: string }).label)).toEqual([
			'Open workspace',
			'Delete project',
		]);
		expect(projectMenu({ onopen: () => {}, onvisit: () => {} }, false)).toHaveLength(1);
	});

	it('floats off ⋯ and picks with the keyboard', async () => {
		const onduplicate = vi.fn();
		await render(ProjectCardMenu, { name: 'salah-daily-tracker', actions: { onopen: () => {}, onduplicate }, live: false });
		const button = page.getByRole('button', { name: 'salah-daily-tracker actions' });
		await button.click();
		await expect.element(button).toHaveAttribute('aria-expanded', 'true');
		await userEvent.keyboard('{ArrowDown}');
		await expect.element(page.getByRole('menuitem', { name: /Duplicate/ })).toHaveFocus();
		await userEvent.keyboard('{Enter}');
		expect(onduplicate).toHaveBeenCalledOnce();
		await expect.element(button).toHaveFocus();
	});
});
