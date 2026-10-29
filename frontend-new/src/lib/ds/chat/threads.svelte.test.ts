import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import ChatMenuFixture from './ChatMenuFixture.svelte';
import RenameField from './RenameField.svelte';
import MoveDialog from './MoveDialog.svelte';
import JumpToLatest from './JumpToLatest.svelte';
import NewDivider from './NewDivider.svelte';
import LoadingOlder from './LoadingOlder.svelte';
import DateDivider from './DateDivider.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Chat responses → Chat management & long threads: the chat itself can be renamed, pinned, moved
// into a project, exported or deleted. Long threads keep your place and tell you when something
// new arrives.

describe('ChatMenu', () => {
	it('renames, pins, moves, exports, shares or deletes the chat', async () => {
		const onpick = vi.fn();
		await render(ChatMenuFixture, { onpick });
		await page.getByRole('button', { name: 'Chat options' }).click();
		await expect.element(page.getByRole('menuitem', { name: /Rename/ })).toHaveTextContent('F2');
		await expect.element(page.getByRole('menuitem', { name: /Export/ })).toHaveTextContent('md · pdf');
		expect(getComputedStyle(page.getByRole('menuitem', { name: 'Delete chat' }).element()).color).toBe(tokenColor('nm-danger'));
		await page.getByRole('menuitem', { name: 'Move to a project' }).click();
		expect(onpick).toHaveBeenCalledWith('move');
	});

	it('offers to unpin a pinned chat', async () => {
		await render(ChatMenuFixture, { onpick: () => {}, pinned: true });
		await page.getByRole('button', { name: 'Chat options' }).click();
		await expect.element(page.getByRole('menuitem', { name: 'Unpin from sidebar' })).toBeVisible();
	});
});

describe('RenameField', () => {
	it('renames in place: Enter saves, Escape cancels', async () => {
		const onsave = vi.fn();
		const oncancel = vi.fn();
		await render(RenameField, { value: 'Food spending', onsave, oncancel });
		const field = page.getByRole('textbox', { name: 'Chat name' });
		await expect.element(field).toHaveFocus();
		expect(getComputedStyle(field.element()).borderTopColor).toBe(tokenColor('nm-accent'));
		await expect.element(page.getByText('↵ save · esc cancel')).toBeVisible();
		await field.fill('Food spending in October');
		await userEvent.keyboard('{Enter}');
		expect(onsave).toHaveBeenCalledWith('Food spending in October');
		await userEvent.keyboard('{Escape}');
		expect(oncancel).toHaveBeenCalledOnce();
	});

	it('does not save an empty name', async () => {
		const onsave = vi.fn();
		await render(RenameField, { value: 'Food', onsave, oncancel: () => {} });
		await page.getByRole('textbox', { name: 'Chat name' }).fill('   ');
		await userEvent.keyboard('{Enter}');
		expect(onsave).not.toHaveBeenCalled();
	});
});

describe('MoveDialog', () => {
	const projects = [
		{ id: 'p1', name: 'budget-keluarga', workspace: 'Keluarga workspace', shared: 'Keluarga' },
		{ id: 'p2', name: 'salah-daily-tracker', workspace: 'Personal' },
		{ id: 'p3', name: 'kos-mawar-tenants', workspace: 'Personal' },
	];

	it('picks a project, says who will see the chat, then moves it', async () => {
		const onmove = vi.fn();
		await page.viewport(1280, 800);
		await render(MoveDialog, { open: true, projects, onmove });
		const dialog = page.getByRole('dialog', { name: 'Move chat to a project' });
		await expect.element(dialog).toBeVisible();
		expect(dialog.element().getBoundingClientRect().width).toBe(440);
		const move = page.getByRole('button', { name: 'Move chat' });
		await expect.element(move).toBeDisabled();
		await page.getByRole('radio', { name: /budget-keluarga/ }).click();
		await expect.element(page.getByText('Members of Keluarga will see this chat. Your memory and money records stay personal.')).toBeVisible();
		expect(getComputedStyle(page.getByText('budget-keluarga').element()).fontFamily).toContain('Geist Mono');
		await page.getByRole('radio', { name: /salah-daily-tracker/ }).click();
		await expect.element(page.getByText(/Members of Keluarga/)).not.toBeInTheDocument();
		await move.click();
		expect(onmove).toHaveBeenCalledWith('p2');
		await expect.element(page.getByRole('dialog')).not.toBeInTheDocument();
	});

	it('filters projects as you search', async () => {
		await render(MoveDialog, { open: true, projects, onmove: () => {} });
		await page.getByRole('searchbox', { name: 'Search projects' }).fill('kos');
		expect(page.getByRole('radio').elements()).toHaveLength(1);
	});
});

describe('Long threads', () => {
	it('jumps to the latest, saying how many replies are new', async () => {
		const onclick = vi.fn();
		await render(JumpToLatest, { count: 2, onclick });
		const button = page.getByRole('button', { name: '2 new replies' });
		expect(getComputedStyle(button.element()).boxShadow).not.toBe('none');
		await button.click();
		expect(onclick).toHaveBeenCalledOnce();
	});

	it('says "Jump to latest" when nothing is new', async () => {
		await render(JumpToLatest, { count: 0, onclick: () => {} });
		await expect.element(page.getByRole('button', { name: 'Jump to latest' })).toBeVisible();
	});

	it('marks what arrived while you were away in mint', async () => {
		await render(NewDivider, { count: 2 });
		const divider = page.getByRole('separator', { name: 'NEW · 2 replies while you were away' });
		await expect.element(divider).toBeVisible();
		expect(getComputedStyle(page.getByText('NEW · 2 replies while you were away').element()).color).toBe(tokenColor('nm-accent'));
	});

	it('loads older messages honestly', async () => {
		await render(LoadingOlder);
		await expect.element(page.getByRole('status')).toHaveTextContent('Loading older messages…');
	});

	it('divides days quietly', async () => {
		const { container } = await render(DateDivider, { label: 'Yesterday' });
		await expect.element(page.getByRole('separator', { name: 'Yesterday' })).toBeVisible();
		const line = container.querySelector('[data-line]')!;
		expect(getComputedStyle(line).backgroundColor).toBe(tokenBackground('nm-border'));
		expect(getComputedStyle(page.getByText('Yesterday').element()).textTransform).toBe('uppercase');
	});
});
