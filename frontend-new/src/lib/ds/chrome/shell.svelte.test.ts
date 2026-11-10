import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import TopBar from './TopBar.svelte';
import CrewPanel from './CrewPanel.svelte';
import Greeting from './Greeting.svelte';
import AccountMenu from './AccountMenu.svelte';
import WorkspaceSwitcher from './WorkspaceSwitcher.svelte';
import ShortcutsDialog from './ShortcutsDialog.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// The app shell around every page (App / Home Vr4nn, App / Account menu t2Bom2, App / Projects /
// Workspace switcher pKiO2): the top bar, the crew column, Home's greeting, the account menu off
// the user row and the workspace switcher off the Build group's chip.

const box = (element: Element) => element.getBoundingClientRect();

describe('TopBar', () => {
	it('names the page, then its date in mono after a slash (u8pcLQ)', async () => {
		await render(TopBar, { title: 'Home', meta: 'Wed, 21 Oct 2026 · 08:14' });
		const title = page.getByText('Home').element();
		const meta = page.getByText('Wed, 21 Oct 2026 · 08:14').element();
		expect(getComputedStyle(title).fontSize).toBe('13px');
		expect(getComputedStyle(title).fontWeight).toBe('500');
		expect(getComputedStyle(meta).fontFamily).toContain('Geist Mono');
		expect(getComputedStyle(meta).color).toBe(tokenColor('nm-text-3'));
		const slash = page.getByText('/', { exact: true }).element();
		expect(slash.closest('[aria-hidden="true"]')).not.toBeNull();
		// Title, slash and date 10 apart.
		expect(Math.round(box(slash).left - box(title).right)).toBe(10);
	});
});

const crew = [
	{ agent: 'nomi' as const, name: 'Nomi', role: 'Talks with you and routes work' },
	{ agent: 'dana' as const, name: 'Dana', role: 'Transactions, budgets, subscriptions', state: 'working' },
	{ name: 'Iris', role: 'Looks into what went wrong' },
];

describe('CrewPanel', () => {
	it('is the crew column: a 52px head with how many are idle, then each member (EXxti)', async () => {
		await render(CrewPanel, { summary: { kind: 'idle', label: '10 idle' }, members: crew });
		const panel = page.getByRole('complementary', { name: 'Your crew' });
		await expect.element(panel).toBeVisible();
		const head = page.getByRole('heading', { name: 'Your crew' }).element().parentElement!;
		expect(box(head).height).toBe(52);
		expect(getComputedStyle(head).paddingLeft).toBe('20px');
		expect(getComputedStyle(head).borderBottomColor).toBe(tokenColor('nm-border'));
		expect(getComputedStyle(page.getByText('10 idle').element()).fontFamily).toContain('Geist Mono');
		const rows = page.getByRole('listitem').elements();
		expect(rows).toHaveLength(3);
		expect(box(rows[0]).height).toBe(52);
		expect(Math.round(box(rows[1]).top - box(rows[0]).bottom)).toBe(2);
		expect(getComputedStyle(rows[0].parentElement!).paddingTop).toBe('8px');
		await expect.element(page.getByText('working')).toBeVisible();
	});

	it('gives a member the design has no mascot for their initial instead', async () => {
		await render(CrewPanel, { summary: { kind: 'idle', label: '3 idle' }, members: crew });
		const row = page.getByRole('listitem').nth(2).element();
		expect(row.querySelector('svg')).toBeNull();
		expect(row.textContent).toContain('I');
	});
});

describe('Greeting', () => {
	it('greets in the display size beside Nomi’s 32px mascot (Z7EVR)', async () => {
		await render(Greeting, { props: { title: 'Morning, Mario.', intro: 'What should we get done? Ask Nomi, or hand it straight to someone on the crew.' } });
		const heading = page.getByRole('heading', { level: 1, name: 'Morning, Mario.' }).element();
		const style = getComputedStyle(heading);
		expect([style.fontSize, style.lineHeight, style.fontWeight, style.letterSpacing]).toEqual(['28px', '36px', '500', '-0.4px']);
		const intro = page.getByText(/What should we get done/).element();
		expect(getComputedStyle(intro).fontSize).toBe('15px');
		expect(getComputedStyle(intro).color).toBe(tokenColor('nm-text-2'));
		expect(Math.round(box(intro).top - box(heading).bottom)).toBe(4);
		const mascot = heading.closest('div')!.parentElement!.querySelector('svg')!;
		expect(box(mascot).width).toBe(32);
		expect(Math.round(box(heading).left - box(mascot).right)).toBe(16);
	});
});

const account = {
	name: 'Mario Prasetyo',
	plan: 'Free · 21.8K / 1M tokens',
	upgradeHref: '/settings/plan',
	settingsHref: '/settings',
	profileHref: '/settings/profile',
	planHref: '/settings/plan',
	theme: 'dark' as const,
	language: 'en',
	languages: [
		{ value: 'en', label: 'English' },
		{ value: 'id', label: 'Bahasa Indonesia' },
	],
	version: 'v0.0.75',
};

describe('AccountMenu', () => {
	it('opens from the user row above it, as the pen draws it (LkJLt)', async () => {
		await render(AccountMenu, { ...account, ontheme: vi.fn(), onlanguage: vi.fn(), onshortcuts: vi.fn(), onlogout: vi.fn() });
		const trigger = page.getByRole('button', { name: 'Mario Prasetyo' });
		await expect.element(trigger).toHaveAttribute('aria-haspopup', 'menu');
		await trigger.click();
		await expect.element(trigger).toHaveAttribute('aria-expanded', 'true');
		const menu = page.getByRole('menu', { name: 'Account' }).element();
		const style = getComputedStyle(menu);
		expect(box(menu).width).toBe(272);
		expect([style.paddingTop, style.rowGap, style.borderTopLeftRadius]).toEqual(['4px', '2px', '10px']);
		expect(style.outlineColor).toBe(tokenColor('nm-border-strong'));
		expect(style.boxShadow).toContain('0px 12px 32px');

		await expect.element(page.getByRole('menuitem', { name: /Settings/ })).toHaveAttribute('href', '/settings');
		await expect.element(page.getByRole('menuitem', { name: /Profile/ })).toHaveAttribute('href', '/settings/profile');
		await expect.element(page.getByRole('menuitem', { name: /Plan & usage/ })).toHaveAttribute('href', '/settings/plan');
		await expect.element(page.getByRole('menuitem', { name: 'Upgrade' })).toHaveAttribute('href', '/settings/plan');
		expect(getComputedStyle(page.getByRole('menuitem', { name: 'Upgrade' }).element()).color).toBe(tokenColor('nm-accent'));
		expect(box(page.getByRole('menuitem', { name: /Settings/ }).element()).height).toBe(30);
		await expect.element(page.getByText('v0.0.75')).toBeVisible();
		await expect.element(page.getByText('⌘ ,')).toBeVisible();
	});

	it('switches the theme in place: three choices, the one in use outlined on bg', async () => {
		const ontheme = vi.fn();
		await render(AccountMenu, { ...account, ontheme, onlanguage: vi.fn(), onlogout: vi.fn() });
		await page.getByRole('button', { name: 'Mario Prasetyo' }).click();
		const dark = page.getByRole('menuitemradio', { name: 'Dark' });
		await expect.element(dark).toHaveAttribute('aria-checked', 'true');
		expect(getComputedStyle(dark.element()).backgroundColor).toBe(tokenBackground('nm-bg'));
		expect([box(dark.element()).width, box(dark.element()).height]).toEqual([26, 22]);
		expect(box(page.getByRole('group', { name: 'Theme' }).element()).height).toBe(36);
		await page.getByRole('menuitemradio', { name: 'Light' }).click();
		expect(ontheme).toHaveBeenCalledWith('light');
		// The menu stays open, so the change can be seen.
		await expect.element(page.getByRole('menu', { name: 'Account' })).toBeVisible();
	});

	it('picks the language from its own list, and goes back', async () => {
		const onlanguage = vi.fn();
		await render(AccountMenu, { ...account, ontheme: vi.fn(), onlanguage, onlogout: vi.fn() });
		await page.getByRole('button', { name: 'Mario Prasetyo' }).click();
		const row = page.getByRole('menuitem', { name: /Language/ });
		expect(box(row.element()).height).toBe(32);
		await expect.element(row).toHaveTextContent('English');
		await row.click();
		const english = page.getByRole('menuitemradio', { name: 'English' });
		await expect.element(english).toHaveAttribute('aria-checked', 'true');
		await expect.element(english).toHaveFocus();
		await userEvent.keyboard('{Escape}');
		await expect.element(page.getByRole('menuitem', { name: /Language/ })).toHaveFocus();
		await page.getByRole('menuitem', { name: /Language/ }).click();
		await userEvent.keyboard('{ArrowDown}{Enter}');
		expect(onlanguage).toHaveBeenCalledWith('id');
		await expect.element(page.getByRole('menu')).not.toBeInTheDocument();
	});

	it('is driven by the keyboard and logs out', async () => {
		const onlogout = vi.fn();
		const onshortcuts = vi.fn();
		await render(AccountMenu, { ...account, ontheme: vi.fn(), onlanguage: vi.fn(), onshortcuts, onlogout });
		const trigger = page.getByRole('button', { name: 'Mario Prasetyo' });
		await trigger.click();
		await expect.element(page.getByRole('menuitem', { name: 'Upgrade' })).toHaveFocus();
		await userEvent.keyboard('{End}');
		await expect.element(page.getByRole('menuitem', { name: /Log out/ })).toHaveFocus();
		await userEvent.keyboard('{ArrowUp}{Enter}');
		expect(onshortcuts).toHaveBeenCalledOnce();
		await expect.element(page.getByRole('menu')).not.toBeInTheDocument();
		await expect.element(trigger).toHaveFocus();
		await trigger.click();
		await userEvent.keyboard('{End}{Enter}');
		expect(onlogout).toHaveBeenCalledOnce();
	});

	it('closes on Escape or a click elsewhere', async () => {
		await render(AccountMenu, { ...account, ontheme: vi.fn(), onlanguage: vi.fn(), onlogout: vi.fn() });
		const trigger = page.getByRole('button', { name: 'Mario Prasetyo' });
		await trigger.click();
		await userEvent.keyboard('{Escape}');
		await expect.element(page.getByRole('menu')).not.toBeInTheDocument();
		await expect.element(trigger).toHaveFocus();
		await trigger.click();
		document.body.dispatchEvent(new PointerEvent('pointerdown', { bubbles: true }));
		await expect.element(page.getByRole('menu')).not.toBeInTheDocument();
	});

	it('leaves out what it can’t do: no Upgrade on the top plan, no shortcuts without a dialog', async () => {
		await render(AccountMenu, { ...account, upgradeHref: undefined, ontheme: vi.fn(), onlanguage: vi.fn(), onlogout: vi.fn() });
		await page.getByRole('button', { name: 'Mario Prasetyo' }).click();
		expect(page.getByRole('menuitem', { name: 'Upgrade' }).elements()).toHaveLength(0);
		expect(page.getByRole('menuitem', { name: /Keyboard shortcuts/ }).elements()).toHaveLength(0);
	});
});

const workspaces = [
	{ id: 'o1', name: 'Mario', tone: 'plain' as const, meta: 'Owner · only you · 1 project', chosen: false },
	{ id: 'o2', name: "Trian's space", tone: 'teal' as const, meta: 'Admin · 3 members · 4 projects', chosen: true },
	{ id: 'o3', name: 'Kopi Kita', tone: 'amber' as const, meta: 'Member · 2 members · 2 projects', chosen: false },
];

describe('WorkspaceSwitcher', () => {
	it('opens from the chip with every workspace, the one for projects checked (t24cE)', async () => {
		await render(WorkspaceSwitcher, { current: "Trian's space", workspaces, settingsHref: '/settings/workspaces/o2', onchoose: vi.fn(), oncreate: vi.fn() });
		const chip = page.getByRole('button', { name: "Projects from Trian's space" });
		await chip.click();
		await expect.element(chip).toHaveAttribute('aria-expanded', 'true');
		const menu = page.getByRole('menu', { name: 'Workspaces' }).element();
		const style = getComputedStyle(menu);
		expect(box(menu).width).toBe(360);
		expect([style.paddingTop, style.rowGap]).toEqual(['6px', '2px']);
		expect(style.boxShadow).toContain('0px 12px 32px');
		expect(getComputedStyle(page.getByText('Workspaces', { exact: true }).element()).textTransform).toBe('uppercase');

		const chosen = page.getByRole('menuitemradio', { name: /Trian's space/ });
		await expect.element(chosen).toHaveAttribute('aria-checked', 'true');
		expect(getComputedStyle(chosen.element()).backgroundColor).toBe(tokenBackground('nm-surface-2'));
		expect(box(chosen.element()).height).toBe(49);
		const tile = chosen.element().querySelector('[data-tile]')!;
		expect([box(tile).width, box(tile).height]).toEqual([32, 32]);
		expect(getComputedStyle(page.getByText('Admin · 3 members · 4 projects').element()).fontFamily).toContain('Geist Mono');
		await expect.element(page.getByRole('menuitemradio', { name: /Kopi Kita/ })).toHaveAttribute('aria-checked', 'false');

		// What a workspace shares, and what stays personal, describes the menu.
		await expect.element(page.getByRole('menu', { name: 'Workspaces' })).toHaveAccessibleDescription(/Shared in a workspace.*Always personal/);
		await expect.element(page.getByRole('menuitem', { name: /Workspace settings & members/ })).toHaveAttribute('href', '/settings/workspaces/o2');
	});

	it('chooses a workspace for projects, or starts a new one', async () => {
		const onchoose = vi.fn();
		const oncreate = vi.fn();
		await render(WorkspaceSwitcher, { current: "Trian's space", workspaces, settingsHref: '/settings/workspaces/o2', onchoose, oncreate });
		await page.getByRole('button', { name: /Projects from/ }).click();
		await page.getByRole('menuitemradio', { name: /Kopi Kita/ }).click();
		expect(onchoose).toHaveBeenCalledWith('o3');
		await expect.element(page.getByRole('menu')).not.toBeInTheDocument();
		await page.getByRole('button', { name: /Projects from/ }).click();
		await userEvent.keyboard('{End}{ArrowUp}{Enter}');
		expect(oncreate).toHaveBeenCalledOnce();
	});

	it('does nothing when the checked one is chosen again', async () => {
		const onchoose = vi.fn();
		await render(WorkspaceSwitcher, { current: "Trian's space", workspaces, onchoose, oncreate: vi.fn() });
		await page.getByRole('button', { name: /Projects from/ }).click();
		await page.getByRole('menuitemradio', { name: /Trian's space/ }).click();
		expect(onchoose).not.toHaveBeenCalled();
	});
});

describe('ShortcutsDialog', () => {
	it('lists the keys the app answers to', async () => {
		await render(ShortcutsDialog, {
			open: true,
			shortcuts: [
				{ label: 'New chat', keys: '⌘K' },
				{ label: 'Close a menu or dialog', keys: 'esc' },
			],
		});
		const dialog = page.getByRole('dialog', { name: 'Keyboard shortcuts' });
		await expect.element(dialog).toBeVisible();
		await expect.element(dialog).toHaveTextContent(/New chat\s*⌘K/);
		expect(getComputedStyle(page.getByText('⌘K').element()).fontFamily).toContain('Geist Mono');
		await page.getByRole('button', { name: 'Done' }).click();
		await expect.element(dialog).not.toBeInTheDocument();
	});
});
