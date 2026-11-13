import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import WorkspaceCard from './WorkspaceCard.svelte';
import CreateWorkspaceCard from './CreateWorkspaceCard.svelte';
import WorkspaceTile from './WorkspaceTile.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Workspaces' pieces (App / Settings / Workspaces l222Mz), measured against the pen.

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();
const card = { name: 'Trian’s space', initial: 'T', tone: 'teal' as const, meta: 'Admin · 3 members', stats: [{ value: 4, label: 'projects' }, { value: 3, label: 'members' }], href: '/settings/workspaces/a', settings: 'Settings', currentTag: 'Current', currentNote: 'Projects come from here', use: 'Use for projects' };

describe('WorkspaceTile', () => {
	it('is the workspace’s initial on its tone, 40 or 56', async () => {
		await render(WorkspaceTile, { initial: 'T', tone: 'teal', size: 56 });
		const tile = page.getByText('T').element();
		// tile-teal is #5cc9c9 (app.css); Tailwind emits its variable only where a class uses it.
		expect([box(tile).width, getComputedStyle(tile).backgroundColor]).toEqual([56, 'rgb(92, 201, 201)']);
	});
});

describe('WorkspaceCard', () => {
	it('is the pen’s 236px card, 18 in: tile, name, role, stats, and Settings under the line', async () => {
		await page.viewport(1512, 862);
		await render(WorkspaceCard, { ...card, current: false });
		const article = page.getByRole('article', { name: 'Trian’s space' }).element();
		expect([box(article).height, getComputedStyle(article).backgroundColor, getComputedStyle(article).outlineColor]).toEqual([236, tokenBackground('nm-surface'), tokenColor('nm-border-strong')]);
		const tile = page.getByText('T', { exact: true }).element();
		expect([box(tile).width, box(tile).left - box(article).left, box(tile).top - box(article).top]).toEqual([40, 18, 18]);
		expect(getComputedStyle(page.getByText('4', { exact: true }).element()).fontFamily).toMatch(/Mono/);
		await expect.element(page.getByRole('link', { name: 'Settings' })).toHaveAttribute('href', '/settings/workspaces/a');
		await expect.element(page.getByRole('button', { name: 'Use for projects' })).toBeVisible();
	});

	it('marks where projects come from: the tag, a stronger edge and the note in place of the button', async () => {
		await render(WorkspaceCard, { ...card, current: true });
		const article = page.getByRole('article', { name: 'Trian’s space' }).element();
		expect(getComputedStyle(article).outlineColor).toBe(tokenColor('nm-text-2'));
		await expect.element(page.getByText('Current')).toBeVisible();
		expect(getComputedStyle(page.getByText('Projects come from here').element()).color).toBe(tokenColor('nm-accent'));
		expect(page.getByRole('button', { name: 'Use for projects' }).elements()).toHaveLength(0);
	});

	it('says Use for projects was asked for', async () => {
		const onuse = vi.fn();
		await render(WorkspaceCard, { ...card, current: false, onuse });
		await page.getByRole('button', { name: 'Use for projects' }).click();
		expect(onuse).toHaveBeenCalledOnce();
	});
});

describe('CreateWorkspaceCard', () => {
	it('is a 236px outlined button with the plus, its title and what it’s for', async () => {
		const onclick = vi.fn();
		await render(CreateWorkspaceCard, { title: 'Create a workspace', body: 'For a team, a client or a side business', onclick });
		const button = page.getByRole('button', { name: /Create a workspace/ }).element();
		expect([box(button).height, getComputedStyle(button).outlineColor]).toEqual([236, tokenColor('nm-border')]);
		await page.getByRole('button', { name: /Create a workspace/ }).click();
		expect(onclick).toHaveBeenCalledOnce();
	});
});
