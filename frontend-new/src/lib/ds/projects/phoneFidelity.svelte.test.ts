import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { CloudOff, FolderInput, SearchX, X } from '@lucide/svelte';
import PhoneProjectSearch from './PhoneProjectSearch.svelte';
import PhoneProjectEmpty from './PhoneProjectEmpty.svelte';
import PhoneFilters from './PhoneFilters.svelte';
import PhoneProjectsHeader from './PhoneProjectsHeader.svelte';
import ProjectCard from './ProjectCard.svelte';
import ProjectCardLoading from './ProjectCardLoading.svelte';
import LoadError from './LoadError.svelte';
import PhoneProjectMenu, { phoneProjectMenu } from './PhoneProjectMenu.svelte';
import PhoneWorkspaceList from './PhoneWorkspaceList.svelte';
import ProjectMoveSheet from './ProjectMoveSheet.svelte';
import DeleteProjectSheet from './DeleteProjectSheet.svelte';
import ProjectComposer from './ProjectComposer.svelte';
import PhoneAskBar from './PhoneAskBar.svelte';
import PhoneProjectDetail from './PhoneProjectDetail.svelte';
import SheetOption from '../overlays/SheetOption.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Flow / Mobile Projects, measured against the pen (phone-fidelity, 50-build-verify.md → Projects):
// each test names the frame it holds the component to.

const style = (element: Element) => getComputedStyle(element);
/** A fully round corner computes huge; read it as "round". */
const corner = (element: Element) => (parseFloat(style(element).borderRadius) >= box(element).height / 2 ? 'round' : style(element).borderRadius);
const box = (element: Element) => element.getBoundingClientRect();

beforeEach(async () => {
	await page.viewport(390, 844);
});
afterEach(() => {
	document.body.innerHTML = '';
});

describe('Find & filter (owsMq)', () => {
	it('searching: a focus stroke, the 20 round clear and Cancel in Sort’s place (oFdFf)', async () => {
		const onsort = vi.fn();
		await render(PhoneProjectSearch, { placeholder: 'Search projects', sortLabel: 'Sort', onsort });
		const field = page.getByRole('searchbox', { name: 'Search projects' });
		await expect.element(page.getByRole('button', { name: 'Cancel' })).not.toBeInTheDocument();
		await field.click();
		const label = field.element().closest('label')!;
		expect([style(label).borderTopColor, style(label).borderTopWidth, style(label).boxShadow]).toEqual([tokenColor('nm-focus'), '1px', 'none']);
		await expect.element(page.getByRole('button', { name: 'Sort' })).not.toBeInTheDocument();
		const cancel = page.getByRole('button', { name: 'Cancel' });
		expect([style(cancel.element()).fontSize, style(cancel.element()).color]).toEqual(['16px', tokenColor('nm-text')]);
		await userEvent.keyboard('kos');
		const clear = page.getByRole('button', { name: 'Clear search' }).element();
		expect([box(clear).width, box(clear).height, corner(clear), style(clear).backgroundColor, style(clear).color]).toEqual([20, 20, 'round', tokenBackground('nm-text-3'), tokenColor('nm-surface-2')]);
		// The field narrows to 299 beside Cancel (16 + 299 + 8 + Cancel + 16).
		expect(Math.round(box(label).width)).toBe(299);
		await page.getByRole('button', { name: 'Clear search' }).click();
		await expect.element(field).toHaveValue('');
		await expect.element(field).toHaveFocus();
		await userEvent.keyboard('kos');
		await cancel.click();
		await expect.element(field).toHaveValue('');
		await expect.element(field).not.toHaveFocus();
		await page.getByRole('button', { name: 'Sort' }).click();
		expect(onsort).toHaveBeenCalledOnce();
	});

	it('no match is the phone’s: no box, the 48 tile, 17 / 600, 14 / 1.45 and a 40 pill with x (OC2PS)', async () => {
		const onclick = vi.fn();
		await render(PhoneProjectEmpty, { icon: SearchX, title: 'No projects match “kasir”', body: 'Search looks at names, descriptions and URLs in Trian’s space.', action: { label: 'Clear search', icon: X, onclick } });
		const root = page.getByText('No projects match “kasir”').element().parentElement!;
		expect([style(root).outlineStyle, style(root).paddingTop, style(root).paddingLeft, style(root).rowGap]).toEqual(['none', '40px', '24px', '10px']);
		const tile = root.firstElementChild!;
		expect([box(tile).width, style(tile).borderRadius, style(tile).backgroundColor, style(tile).color]).toEqual([48, '14px', tokenBackground('nm-surface-2'), tokenColor('nm-text-2')]);
		expect(box(tile.querySelector('svg')!).width).toBe(22);
		const title = page.getByText('No projects match “kasir”').element();
		expect([style(title).fontSize, style(title).fontWeight, box(title).top - box(root).top]).toEqual(['17px', '600', 98]);
		const body = page.getByText(/Search looks at/).element();
		expect([style(body).fontSize, style(body).lineHeight, style(body).color]).toEqual(['14px', '20px', tokenColor('nm-text-3')]);
		const clear = page.getByRole('button', { name: 'Clear search' }).element();
		expect([box(clear).height, corner(clear), style(clear).fontSize, style(clear).fontWeight]).toEqual([40, 'round', '14px', '500']);
		expect(clear.querySelector('svg')).not.toBeNull();
		await page.getByRole('button', { name: 'Clear search' }).click();
		expect(onclick).toHaveBeenCalledOnce();
	});

	it('the filters’ row is 46 tall (J8au5J)', async () => {
		await render(PhoneFilters, { label: 'Show', value: 'all', options: [{ value: 'all', label: 'All', count: 1 }], onchange: () => {} });
		expect(box(page.getByRole('radiogroup', { name: 'Show' }).element()).height).toBe(46);
	});

	it('the filters’ row does not shrink inside a flex-col scroll container', async () => {
		const wrapper = document.createElement('div');
		wrapper.style.display = 'flex';
		wrapper.style.flexDirection = 'column';
		wrapper.style.height = '200px';
		wrapper.style.overflowY = 'auto';
		document.body.appendChild(wrapper);

		try {
			await render(PhoneFilters, {
				target: wrapper,
				props: {
					label: 'Filter inbox',
					value: 'all',
					options: [
						{ value: 'all', label: 'All', count: 3 },
						{ value: 'needs', label: 'Needs you', count: 3 },
					],
					onchange: () => {},
				},
			});

			const tall = document.createElement('div');
			tall.style.height = '800px';
			tall.style.flexShrink = '0';
			wrapper.appendChild(tall);

			expect(box(page.getByRole('radiogroup', { name: 'Filter inbox' }).element()).height).toBe(46);
		} finally {
			wrapper.remove();
		}
	});

	it('the sort sheet’s check is 18 (i7GQct)', async () => {
		await render(SheetOption, { label: 'Updated', icon: FolderInput, chosen: true, onclick: () => {} });
		const svgs = page.getByRole('button', { name: 'Updated' }).element().querySelectorAll('svg');
		expect(box(svgs[svgs.length - 1]).width).toBe(18);
	});
});

describe('Header and cards (etEGD, zz1kw)', () => {
	it('the workspace pill’s initial is sans 12 / 600 (AXKpW)', async () => {
		await render(PhoneProjectsHeader, { title: 'Projects', workspace: "Trian's space", labels: { workspace: 'Workspace', members: 'People', invite: 'Invite', new: 'New' }, onworkspace: () => {} });
		const initial = page.getByText('T', { exact: true }).element();
		expect([style(initial).fontFamily.startsWith('Geist Mono'), style(initial).fontSize, style(initial).fontWeight]).toEqual([false, '12px', '600']);
	});

	it('the card’s name and line are 3 apart, the line at 13 / 1.35 (fbJBa)', async () => {
		await render(ProjectCard, { phone: true, name: 'invoice-sinar-jaya', description: 'Invoices and payment tracking for PT Sinar Jaya', tone: 'kala', status: 'live', footer: '2 chats', href: '/p', onmore: () => {} });
		const line = page.getByText('Invoices and payment tracking for PT Sinar Jaya').element();
		expect([style(line.parentElement!).rowGap, style(line).lineHeight]).toEqual(['3px', '18px']);
	});

	it('the loading card is 194 tall with the card’s border-strong stroke, its bars as drawn (V6bMR)', async () => {
		await render(ProjectCardLoading, { phone: true });
		const card = page.getByRole('status', { name: 'Loading a project' }).element();
		expect([box(card).height, style(card).outlineColor]).toEqual([194, tokenColor('nm-border-strong')]);
		const [name, line] = [...card.firstElementChild!.lastElementChild!.children];
		expect([box(name).height, box(line).height, style(name).borderRadius, box(line).top - box(name).bottom]).toEqual([12, 10, '4px', 3]);
		const state = card.children[1];
		expect([box(state).height, state.children.length]).toEqual([46, 2]);
	});

	it('couldn’t load: 40 above the tile with 14 corners, 17 title, code at 12 / 16, the line at 1.45 (tDuEM)', async () => {
		await render(LoadError, { phone: true, reason: '503 from Nomi’s server · 09:14', onretry: () => {} });
		const root = page.getByRole('alert').element();
		expect([style(root).paddingTop, style(root).rowGap]).toEqual(['40px', '10px']);
		expect([style(root.firstElementChild!).borderRadius, style(root.firstElementChild!).backgroundColor]).toEqual(['14px', tokenBackground('nm-danger-bg')]);
		expect(style(page.getByText('Couldn’t load your projects').element()).fontSize).toBe('17px');
		const code = page.getByText('503 from Nomi’s server · 09:14').element();
		expect([style(code).fontSize, style(code).lineHeight]).toEqual(['12px', '16px']);
		expect(style(page.getByText(/only this list failed/).element()).lineHeight).toBe('20px');
		void CloudOff;
	});
});

describe('Card sheet (Card menu M4p0u)', () => {
	const actions = { onopen: () => {}, onvisit: () => {}, onrename: () => {}, onduplicate: () => {}, onmove: () => {}, onshare: () => {}, onexport: () => {}, onsettings: () => {}, ontakedown: () => {}, ondelete: () => {} };
	it('groups as the phone pen does, its icons and chevrons, under inset separators (R4a7xS)', async () => {
		const rows = phoneProjectMenu(actions, true).map((entry) => (entry === 'separator' ? '—' : `${entry.label}${entry.more ? ' ›' : ''}${entry.danger ? ' !' : ''}`));
		expect(rows).toEqual(['Open workspace', 'Visit live site', '—', 'Rename', 'Duplicate', 'Move to workspace… ›', 'Share… ›', 'Export repo', 'Settings ›', '—', 'Take down', 'Delete project !']);
		const onpick = vi.fn();
		await render(PhoneProjectMenu, { actions, live: true, onpick });
		const icon = (name: string) => page.getByRole('button', { name }).element().querySelector('svg')!.getAttribute('class') ?? '';
		expect([icon('Open workspace'), icon('Visit live site'), icon('Take down')].map((name) => name.split(' ').find((one) => /^lucide-(?!icon)/.test(one)))).toEqual(['lucide-app-window', 'lucide-arrow-up-right', 'lucide-power']);
		const separator = document.querySelector('span[aria-hidden="true"].flex')!;
		expect([box(separator).height, box(separator.firstElementChild!).width]).toEqual([9, 350]);
		expect(page.getByRole('button', { name: 'Move to workspace…' }).element().querySelectorAll('svg').length).toBe(2);
		await page.getByRole('button', { name: 'Rename' }).click();
		expect(onpick).toHaveBeenCalledOnce();
	});
});

describe('Move (WbD1H) and Delete (Z6OsC)', () => {
	const workspaces = [
		{ id: 'o2', name: "Trian's space", meta: 'current · 3 members', tone: 'plain' as const, members: 3, current: true },
		{ id: 'o3', name: 'Kopi Kita', meta: 'Admin · 5 members', tone: 'plain' as const, members: 5 },
	];
	it('titles the sheet “Move <name>”, says “current” once, and draws the 68 cards (rbozn, KEIJW, I5UHw)', async () => {
		await render(ProjectMoveSheet, { open: true, name: 'salah-tracker', chats: 2, workspaces, onmove: () => {} });
		await expect.element(page.getByRole('dialog', { name: 'Move salah-tracker' })).toBeVisible();
		await expect.element(page.getByText('current · 3 members', { exact: true })).toBeVisible();
		const card = page.getByRole('radio', { name: /Kopi Kita/ }).element();
		const tile = card.firstElementChild!;
		expect([box(card).height, style(card).paddingTop, box(tile).width, style(tile).borderRadius]).toEqual([68, '14px', 36, '9px']);
		expect(style(page.getByText('Kopi Kita').element()).fontWeight).toBe('500');
		expect(card.querySelector('circle')!.getAttribute('stroke-width')).toBe('1.5');
		const group = card.parentElement!.parentElement!;
		expect(style(group).paddingTop).toBe('4px');
	});

	it('deletes with what the backend does to a project, at 15 / 1.4 (AHDzi, v81Hi)', async () => {
		await render(DeleteProjectSheet, { open: true, name: 'salah-tracker', ondelete: () => {} });
		const body = page.getByText(/The live URL stops working and the database is removed/).element();
		expect(body.textContent).toBe('The live URL stops working and the database is removed. Its code goes too, for good: export it first if you want a copy. Its chat stays.');
		expect(style(body).lineHeight).toBe('21px');
	});
});

describe('Workspaces sheet (Workspaces & people F2Tuh)', () => {
	it('draws the workspaces as cards, the chosen one filled, and the boundary under them (a9NrP, C8evkZ)', async () => {
		const onchoose = vi.fn();
		await render(PhoneWorkspaceList, { workspaces: [{ id: 'a', name: 'Mario', meta: 'only you' }, { id: 'b', name: "Trian's space", meta: 'Admin · 3 members' }], chosen: 'b', onchoose });
		const chosen = page.getByRole('button', { name: /Trian/ }).element();
		const other = page.getByRole('button', { name: /Mario/ }).element();
		expect([box(chosen).height, style(chosen).borderRadius, style(chosen).backgroundColor, style(other).backgroundColor]).toEqual([64, '14px', tokenBackground('nm-surface-2'), 'rgba(0, 0, 0, 0)']);
		const tile = chosen.firstElementChild!;
		expect([box(tile).width, style(tile).borderRadius, style(tile).backgroundColor]).toEqual([36, '9px', tokenBackground('nm-bg')]);
		expect(style(page.getByText("Trian's space").element()).fontWeight).toBe('500');
		await expect.element(page.getByText('Shared in a workspace')).toBeVisible();
		await expect.element(page.getByText('Always personal')).toBeVisible();
		const boundary = page.getByText('Shared in a workspace').element().closest('.rounded-lg')!;
		expect([style(boundary).backgroundColor, style(boundary).padding, style(boundary).borderRadius]).toEqual([tokenBackground('nm-bg'), '14px', '12px']);
		await page.getByRole('button', { name: /Mario/ }).click();
		expect(onchoose).toHaveBeenCalledWith('a');
	});
});

describe('New project (H4ngT2)', () => {
	it('labels its chips in text-3 (CvjGu)', async () => {
		await render(ProjectComposer, { phone: true, domain: 'trian.space', onbuild: () => {} });
		expect(style(page.getByText('git repo').element()).color).toBe(tokenColor('nm-text-3'));
	});
});

describe('Project detail (Detail panel G4Mwvd)', () => {
	it('asks Koda from the pen’s ask bar: 66 under a top stroke, the 52 bar, the mascot, send once typed (kB2Dt)', async () => {
		const onsend = vi.fn();
		await render(PhoneAskBar, { placeholder: 'Ask Koda to change something', onsend });
		const field = page.getByRole('textbox', { name: 'Ask Koda to change something' });
		const wrap = field.element().closest('form')!;
		const bar = field.element().parentElement!;
		expect([box(wrap).height, style(wrap).borderTopColor, box(bar).height, corner(bar), style(bar).backgroundColor, style(bar).boxShadow === 'none']).toEqual([66, tokenColor('nm-border'), 52, 'round', tokenBackground('nm-surface'), false]);
		expect(bar.querySelector('svg')).not.toBeNull();
		await expect.element(page.getByRole('button', { name: 'Send' })).not.toBeInTheDocument();
		await field.fill('Add a Maghrib reminder');
		const send = page.getByRole('button', { name: 'Send' }).element();
		expect([box(send).width, style(send).backgroundColor]).toEqual([40, tokenBackground('nm-surface-2')]);
		await userEvent.keyboard('{Enter}');
		expect(onsend).toHaveBeenCalledWith('Add a Maghrib reminder');
		await expect.element(field).toHaveValue('');
	});

	it('marks Koda at work with the warning dot', async () => {
		await render(PhoneProjectDetail, {
			name: 'salah-tracker',
			tone: 'kala',
			status: 'Koda is changing it · #15 stays live',
			dot: 'working',
			workspaceHref: '/p',
			parts: { code: '1', url: 'a', data: 'b', deploys: 'c', members: 'd' },
			chats: [],
			backLabel: 'Projects',
			onback: () => {},
		});
		const dot = page.getByText('Koda is changing it · #15 stays live').element().previousElementSibling!;
		expect(style(dot).backgroundColor).toBe(tokenBackground('nm-warning'));
	});
});
