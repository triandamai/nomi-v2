import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import ProjectCard from './ProjectCard.svelte';
import ProjectCardLoading from './ProjectCardLoading.svelte';
import ProjectCardMenu, { projectMenu } from './ProjectCardMenu.svelte';
import ProjectMenuPanel from './ProjectMenuPanel.svelte';
import PhoneProjectsHeader from './PhoneProjectsHeader.svelte';
import PhoneProjectSearch from './PhoneProjectSearch.svelte';
import PhoneFilters from './PhoneFilters.svelte';
import ProjectComposer from './ProjectComposer.svelte';
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
	it('is selected by a click on the card or its name, not on its controls', async () => {
		const onselect = vi.fn();
		const onaction = vi.fn();
		await render(ProjectCard, { ...base, status: 'stopped', detail: 'nothing deployed · files kept', onaction, onselect });
		await page.getByText('nothing deployed · files kept').click();
		expect(onselect).toHaveBeenCalledOnce();
		await page.getByRole('button', { name: 'Resume' }).click();
		expect(onaction).toHaveBeenCalledOnce();
		expect(onselect).toHaveBeenCalledOnce();
		const name = page.getByRole('button', { name: 'salah-daily-tracker' });
		await expect.element(name).toHaveAttribute('aria-pressed', 'false');
		name.element().focus();
		await userEvent.keyboard('{Enter}');
		expect(onselect).toHaveBeenCalledTimes(2);
		// The name keeps its place and size as a button.
		expect(box(name.element()).height).toBe(17);
	});

	it('says what Koda is doing in its own words when it’s changing a live app or deploying', async () => {
		await render(ProjectCard, { ...base, status: 'building', statusLabel: 'Koda is deploying', meta: '00:24', detail: 'deploy #1 · building 3 files', footer: '1 chat · just now' });
		await expect.element(page.getByText('Koda is deploying')).toBeInTheDocument();
		await expect.element(page.getByText('00:24')).toBeInTheDocument();
		await expect.element(page.getByText('Koda is building')).not.toBeInTheDocument();
	});

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

describe('ProjectCard on a phone (Mobile / Project Card)', () => {
	it('is the column’s width: 14 corners, a 40 tile, the name at 16, ⋯ opening the sheet, the action a 40 pill', async () => {
		await page.viewport(390, 844);
		const onmore = vi.fn();
		const onaction = vi.fn();
		await render(ProjectCard, { ...base, name: 'invoice-sinar-jaya', status: 'failed', meta: 'deploy #6', detail: 'build error in src/lib/pdf.ts · #5 still live', phone: true, onmore, onaction });
		const card = page.getByRole('article', { name: 'invoice-sinar-jaya' }).element();
		expect([Math.round(box(card).width), style(card).borderRadius, style(card).paddingTop]).toEqual([390, '14px', '16px']);
		expect(box(card.querySelector('span')!).width).toBe(40);
		expect(style(page.getByRole('heading', { name: 'invoice-sinar-jaya' }).element()).fontSize).toBe('16px');
		const zone = page.getByTestId('status-zone').element();
		expect([style(zone).borderRadius, style(zone).paddingTop, style(zone).backgroundColor]).toEqual(['10px', '10px', tokenBackground('nm-danger-bg')]);
		const fix = page.getByRole('button', { name: 'Ask Koda to fix' });
		expect(box(fix.element()).height).toBe(40);
		await fix.click();
		expect(onaction).toHaveBeenCalledOnce();
		await page.getByRole('button', { name: /invoice-sinar-jaya/ }).click();
		expect(onmore).toHaveBeenCalledOnce();
	});
});


describe('Projects on a phone: the header, the search and the filters', () => {
	it('is the pen’s header: the workspace pill, people, Invite, the large title and New', async () => {
		await page.viewport(390, 844);
		const onworkspace = vi.fn();
		const onnew = vi.fn();
		await render(PhoneProjectsHeader, {
			title: 'Projects',
			workspace: "Trian's space",
			members: 3,
			labels: { workspace: 'Workspace: Trian’s space', members: 'People', invite: 'Invite people', new: 'New' },
			onworkspace,
			oninvite: () => {},
			onnew,
		});
		const pill = page.getByRole('button', { name: 'Workspace: Trian’s space' }).element();
		expect([box(pill).height, style(pill).backgroundColor]).toEqual([36, tokenBackground('nm-surface-2')]);
		expect(style(page.getByRole('heading', { level: 1, name: 'Projects' }).element()).fontSize).toBe('32px');
		const create = page.getByRole('button', { name: 'New' }).element();
		expect([box(create).height, style(create).backgroundColor]).toEqual([36, tokenBackground('nm-accent')]);
		await page.getByRole('button', { name: 'New' }).click();
		await page.getByRole('button', { name: 'Workspace: Trian’s space' }).click();
		expect([onnew, onworkspace].map((fn) => fn.mock.calls.length)).toEqual([1, 1]);
		await expect.element(page.getByRole('button', { name: 'Invite people' })).toBeVisible();
	});

	it('searches in a 40 field with Sort beside it', async () => {
		const onsort = vi.fn();
		await render(PhoneProjectSearch, { placeholder: 'Search projects', sortLabel: 'Sort', onsort });
		const field = page.getByRole('searchbox', { name: 'Search projects' }).element();
		expect([box(field.closest('label')!).height, style(field).fontSize]).toEqual([40, '16px']);
		await page.getByRole('button', { name: 'Sort' }).click();
		expect(onsort).toHaveBeenCalledOnce();
	});

	it('filters with 34 chips, the chosen one filled, the arrows moving the choice', async () => {
		const onchange = vi.fn();
		await render(PhoneFilters, { label: 'Show', value: 'all', options: [{ value: 'attention', label: 'Needs you', count: 1 }, { value: 'all', label: 'All', count: 4 }, { value: 'live', label: 'Live', count: 2 }], onchange });
		const all = page.getByRole('radio', { name: /All/ });
		await expect.element(all).toHaveAttribute('aria-checked', 'true');
		expect([box(all.element()).height, style(all.element()).backgroundColor]).toEqual([34, tokenBackground('nm-text')]);
		await page.getByRole('radio', { name: /Live/ }).click();
		expect(onchange).toHaveBeenLastCalledWith('live');
		(all.element() as HTMLElement).focus();
		await userEvent.keyboard('{ArrowLeft}');
		expect(onchange).toHaveBeenLastCalledWith('attention');
	});
});

describe('ProjectComposer on a phone (New project sheet)', () => {
	it('sets the text at 16 in a 14-round box, wraps the chips, and starts with the inverse pill', async () => {
		await page.viewport(390, 844);
		const onbuild = vi.fn();
		await render(ProjectComposer, { phone: true, value: 'Order tracker for my laundry', domain: 'trian.space', database: 'for orders', onbuild });
		const field = page.getByRole('textbox', { name: 'Describe an app to build' }).element();
		expect([style(field).fontSize, style(field.parentElement!).borderRadius]).toEqual(['16px', '14px']);
		const start = page.getByRole('button', { name: 'Start building' }).element();
		expect([box(start).height, style(start).backgroundColor]).toEqual([40, tokenBackground('nm-text')]);
		await page.getByRole('button', { name: 'Start building' }).click();
		expect(onbuild).toHaveBeenCalledWith('Order tracker for my laundry');
	});
});
