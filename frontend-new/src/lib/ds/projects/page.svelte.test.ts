import { beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { createRawSnippet } from 'svelte';
import { GitFork } from '@lucide/svelte';
import ProjectTopBar from './ProjectTopBar.svelte';
import ProjectPageHeader from './ProjectPageHeader.svelte';
import ProjectComposer from './ProjectComposer.svelte';
import FilterTab from './FilterTab.svelte';
import FilterTabs from './FilterTabs.svelte';
import ProjectSearch from './ProjectSearch.svelte';
import SortButton from './SortButton.svelte';
import SortMenu from './SortMenu.svelte';
import ProjectToolbar from './ProjectToolbar.svelte';
import ProjectSectionHead from './ProjectSectionHead.svelte';
import RemixCard from './RemixCard.svelte';
import { tokenColor } from '../test-utils';

// Design System / Projects → Page parts & grid (URIB6 / RexQ5): the top bar, the page header, the
// composer and the toolbar's parts, read off the pen frames named in each test.

const style = (element: Element) => getComputedStyle(element);
const box = (element: Element) => element.getBoundingClientRect();

// Wide enough for the grid's two columns and the widest dialog, as on a desktop.
beforeEach(async () => {
	await page.viewport(1280, 800);
});

const workspaces = [
	{ id: 'w1', name: 'Mario', tone: 'plain' as const, meta: 'only you · 1 project', tag: 'Personal', chosen: false },
	{ id: 'w2', name: "Trian's space", tone: 'teal' as const, meta: 'Admin · 3 members · 4 projects', chosen: true },
];
const members = [
	{ initials: 'T', name: 'Trian', tone: 'teal' as const },
	{ initials: 'MP', name: 'Mario Prasetyo', tone: 'plain' as const },
	{ initials: 'SR', name: 'Siti Rahma', tone: 'orange' as const },
];

describe('ProjectTopBar (FRyfm)', () => {
	it('is 52 tall, 24 in, with the line under it drawn inside', async () => {
		await render(ProjectTopBar, { workspaces, onchoose: () => {}, oncreate: () => {}, members });
		const bar = page.getByRole('banner').element();
		expect(box(bar).height).toBe(52);
		expect(style(bar).paddingLeft).toBe('24px');
		expect(style(bar).borderBottomWidth).toBe('0px');
		const crumb = page.getByRole('button', { name: "Workspace: Trian's space" }).element();
		expect(box(crumb).height).toBe(28);
		expect(box(crumb).top - box(bar).top).toBe(12);
		expect(style(crumb).borderTopColor).toBe(tokenColor('nm-border-strong'));
		await expect.element(page.getByText('Projects', { exact: true })).toBeInTheDocument();
	});

	it('opens the workspace switcher from the crumb', async () => {
		const onchoose = vi.fn();
		await render(ProjectTopBar, { workspaces, onchoose, oncreate: () => {}, members });
		const crumb = page.getByRole('button', { name: "Workspace: Trian's space" });
		await crumb.click();
		await expect.element(crumb).toHaveAttribute('aria-expanded', 'true');
		await page.getByRole('menuitemradio', { name: /Mario/ }).click();
		expect(onchoose).toHaveBeenCalledWith('w1');
	});

	it('shows who is in the workspace, how many, and invites', async () => {
		const oninvite = vi.fn();
		await render(ProjectTopBar, { workspaces, onchoose: () => {}, oncreate: () => {}, members, oninvite });
		const people = page.getByRole('group', { name: 'People in this workspace' }).element();
		const avatars = [...people.children];
		expect(avatars).toHaveLength(3);
		expect(box(avatars[0]).width).toBe(24);
		expect(box(avatars[1]).left - box(avatars[0]).left).toBe(18);
		expect(avatars[2].getAttribute('title')).toBe('Siti Rahma');
		expect(style(page.getByText('3 members').element()).fontFamily).toContain('Geist Mono');
		await page.getByRole('button', { name: 'Invite' }).click();
		expect(oninvite).toHaveBeenCalledOnce();
	});
});

describe('ProjectPageHeader (lAF6T)', () => {
	it('is the page title over its lede, 6 apart, two columns wide', async () => {
		await render(ProjectPageHeader);
		const title = page.getByRole('heading', { level: 1, name: 'Projects' }).element();
		const lede = page.getByText(/Apps Koda builds and ships for you/).element();
		expect(box(title.parentElement!).width).toBe(784);
		expect(box(lede).top - box(title).bottom).toBe(6);
		expect(style(lede).lineHeight).toBe('23px');
	});
});

describe('ProjectComposer without a domain', () => {
	it('leaves the domain chip out where none is set', async () => {
		await render(ProjectComposer, { domain: '', onbuild: () => {} });
		expect(page.getByText(/^\*\./).elements()).toHaveLength(0);
		await expect.element(page.getByText('git repo')).toBeVisible();
	});
});

describe('ProjectComposer (MDdO8, PV1lT)', () => {
	it('is 98 tall: a 48 input row and a 50 row of what the project gets', async () => {
		await render(ProjectComposer, { domain: 'trian.space', onbuild: () => {} });
		const form = page.getByRole('form', { name: 'Describe an app to build' }).element();
		expect(box(form).height).toBe(98);
		expect(box(form).width).toBe(784);
		expect(style(form).outlineColor).toBe(tokenColor('nm-border-strong'));
		const provisions = page.getByRole('list', { name: 'What the project gets' }).element();
		expect([...provisions.children].map((chip) => chip.textContent?.trim())).toEqual(['git repo', '*.trian.space', 'database · optional']);
		expect(box(provisions.children[0]).height).toBe(24);
		await expect.element(page.getByRole('textbox', { name: 'Describe an app to build' })).toHaveAttribute('placeholder', 'Describe an app to build, e.g. a booking page for my barbershop…');
	});

	it('turns accent while typing, and lights the database when Koda says what it is for', async () => {
		await render(ProjectComposer, { domain: 'trian.space', database: 'for orders', value: 'Order tracker for my kiloan laundry', onbuild: () => {} });
		const form = page.getByRole('form', { name: 'Describe an app to build' }).element();
		expect(style(form).outlineColor).toBe(tokenColor('nm-accent'));
		const chip = page.getByText('database · for orders').element().parentElement!;
		expect(style(chip).outlineColor).toBe(tokenColor('nm-accent'));
		expect(style(page.getByText('database · for orders').element()).color).toBe(tokenColor('nm-text'));
	});

	it('builds on Enter or the button, and imports from GitHub', async () => {
		const onbuild = vi.fn();
		const onimport = vi.fn();
		await render(ProjectComposer, { domain: 'trian.space', onbuild, onimport });
		const field = page.getByRole('textbox', { name: 'Describe an app to build' });
		await page.getByRole('button', { name: 'Start building' }).click();
		expect(onbuild).not.toHaveBeenCalled();
		await field.fill('A booking page');
		await userEvent.keyboard('{Enter}');
		expect(onbuild).toHaveBeenCalledWith('A booking page');
		await page.getByRole('button', { name: 'Import from GitHub' }).click();
		expect(onimport).toHaveBeenCalledOnce();
	});
});

describe('FilterTab (Tgc5G, kTRZ9) and FilterTabs', () => {
	it('is 26 tall, 10 in; the open one on bg with a strong stroke', async () => {
		await render(FilterTab, { label: 'All', count: 4, active: true });
		const tab = page.getByText('All').element().parentElement!;
		expect(box(tab).height).toBe(26);
		expect(style(tab).paddingLeft).toBe('9px');
		expect(style(tab).borderTopColor).toBe(tokenColor('nm-border-strong'));
		expect(style(page.getByText('All').element()).fontWeight).toBe('400');
	});

	it('is a radio group the arrow keys move through', async () => {
		const onchange = vi.fn();
		await render(FilterTabs, {
			label: 'Show projects',
			options: [
				{ value: 'all', label: 'All', count: 4 },
				{ value: 'live', label: 'Live', count: 2 },
			],
			onchange,
		});
		const all = page.getByRole('radio', { name: /All/ });
		await expect.element(all).toHaveAttribute('aria-checked', 'true');
		all.element().focus();
		await userEvent.keyboard('{ArrowRight}');
		expect(onchange).toHaveBeenCalledWith('live');
		await expect.element(page.getByRole('radio', { name: /Live/ })).toHaveFocus();
	});
});

describe('ProjectSearch (m5wmYk, m3EFzX)', () => {
	it('is 220 by 30, quiet when empty and accent once it filters, with a clear', async () => {
		await render(ProjectSearch, { value: 'kasir' });
		const field = page.getByRole('searchbox', { name: 'Search projects' }).element();
		const frame = field.parentElement!;
		expect(box(frame).width).toBe(220);
		expect(box(frame).height).toBe(30);
		expect(style(frame).borderTopColor).toBe(tokenColor('nm-accent'));
		await page.getByRole('button', { name: 'Clear search' }).click();
		await expect.element(page.getByRole('searchbox')).toHaveValue('');
		await expect.element(page.getByRole('searchbox')).toHaveFocus();
		field.blur();
		expect(style(frame).borderTopColor).toBe(tokenColor('nm-border'));
		await expect.element(page.getByRole('button', { name: 'Clear search' })).not.toBeInTheDocument();
	});
});

describe('SortButton (MYatL) and SortMenu (MLYM5)', () => {
	it('names the order and opens the menu with it checked', async () => {
		const onchange = vi.fn();
		await render(SortButton, { value: 'updated', onchange });
		const button = page.getByRole('button', { name: 'Sort by Updated' });
		expect(box(button.element()).height).toBe(30);
		await button.click();
		await expect.element(page.getByRole('menuitemradio', { name: 'Updated' })).toHaveAttribute('aria-checked', 'true');
		await expect.element(page.getByRole('menuitemradio', { name: 'Updated' })).toHaveFocus();
		await userEvent.keyboard('{ArrowDown}');
		await expect.element(page.getByRole('menuitemradio', { name: 'Name' })).toHaveFocus();
		await userEvent.keyboard('{Enter}');
		expect(onchange).toHaveBeenCalledWith('name');
		await expect.element(page.getByRole('menu')).not.toBeInTheDocument();
	});

	it('is 200 wide, its title 24 tall, rows 30 and 2 apart', async () => {
		await render(SortMenu, { value: 'updated', onchoose: () => {} });
		const menu = page.getByRole('menu', { name: 'Sort by' }).element();
		expect(box(menu).width).toBe(200);
		expect(box(menu).height).toBe(160);
		const rows = page.getByRole('menuitemradio').elements();
		expect(rows).toHaveLength(4);
		expect(box(rows[0]).height).toBe(30);
		expect(box(rows[0]).top - box(menu).top).toBe(30);
		expect(box(rows[1]).top - box(rows[0]).bottom).toBe(2);
	});
});

describe('ProjectToolbar (TnDHg)', () => {
	it('filters on the left; search and sort at the end, 8 apart', async () => {
		await render(ProjectToolbar, { counts: { all: 4, live: 2, building: 1, attention: 1 } });
		const tabs = page.getByRole('radiogroup', { name: 'Show projects' }).element();
		expect(box(tabs).height).toBe(30);
		expect(page.getByRole('radio').elements().map((radio) => radio.textContent?.replace(/\s+/g, ' ').trim())).toEqual(['All 4', 'Live 2', 'Building 1', 'Needs attention 1']);
		const search = page.getByRole('searchbox').element().parentElement!;
		const sort = page.getByRole('button', { name: /Sort by/ }).element();
		expect(box(sort).left - box(search).right).toBe(8);
	});
});

describe('ProjectSectionHead (TYcRO)', () => {
	it('is a 40 tall head: icon and title over a line, the link at the bottom right', async () => {
		await render(ProjectSectionHead, {
			icon: GitFork,
			title: 'Remix from the community',
			description: 'Start from an app someone already built.',
			link: { label: 'Browse showcase', href: '/showcase' },
		});
		const heading = page.getByRole('heading', { name: 'Remix from the community' }).element();
		expect(style(heading).fontWeight).toBe('600');
		const head = heading.parentElement!.parentElement!;
		expect(box(head).height).toBe(40);
		const link = page.getByRole('link', { name: /Browse showcase/ }).element();
		expect(box(link).bottom).toBe(box(head).bottom);
	});
});

describe('RemixCard (htwHt)', () => {
	it('is a 384 card: a 104 preview over its maker and Remix', async () => {
		const onremix = vi.fn();
		const preview = createRawSnippet(() => ({ render: () => '<div>Room 1</div>' }));
		await render(RemixCard, { name: 'Kos Melati rent tracker', maker: 'Bu Sari · Yogyakarta · 214 remixes', crew: ['dana', 'kala'], tone: 'dana', preview, onremix });
		const card = page.getByRole('article', { name: 'Kos Melati rent tracker' }).element();
		expect(box(card).width).toBe(384);
		expect(box(card).height).toBe(169);
		expect(style(card).outlineColor).toBe(tokenColor('nm-border-strong'));
		await page.getByRole('button', { name: 'Remix Kos Melati rent tracker' }).click();
		expect(onremix).toHaveBeenCalledOnce();
	});
});
