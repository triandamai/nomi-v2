import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { Bell } from '@lucide/svelte';
import ProjectRow from './ProjectRow.svelte';
import SuggestionRow from './SuggestionRow.svelte';
import EmptyState from './EmptyState.svelte';
import SkeletonRow from './SkeletonRow.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

describe('ProjectRow', () => {
	it('shows the app’s tile, what it is, whether it is live, where, and when it changed', async () => {
		await render(ProjectRow, {
			href: '/projects/salah',
			name: 'salah-daily-tracker',
			description: 'Daily salah tracker with countdown and streaks',
			tone: 'nomi',
			status: { kind: 'live', label: 'Live' },
			url: 'salah-daily-tracker-0ab8f8.trian.space',
			updated: '2d',
		});
		const link = page.getByRole('link', { name: /salah-daily-tracker/ });
		await expect.element(link).toHaveAttribute('href', '/projects/salah');
		const tile = page.getByText('S', { exact: true }).element();
		expect(getComputedStyle(tile.parentElement!).backgroundColor).toBe(tokenBackground('crew-nomi-bg'));
		expect(getComputedStyle(tile).color).toBe(tokenColor('crew-nomi-ink'));
		expect(getComputedStyle(tile).fontFamily).toContain('Fraunces');
		expect(getComputedStyle(page.getByText('salah-daily-tracker-0ab8f8.trian.space').element()).fontFamily).toContain('Geist Mono');
	});
});

describe('SuggestionRow', () => {
	it('offers something to ask, and says who it goes to', async () => {
		const onclick = vi.fn();
		await render(SuggestionRow, { agent: 'dana', label: "Summarize this week's spending", route: '→ dana', onclick });
		const button = page.getByRole('button', { name: /Summarize this week's spending/ });
		await button.click();
		expect(onclick).toHaveBeenCalledOnce();
		expect(getComputedStyle(page.getByText('→ dana').element()).fontFamily).toContain('Geist Mono');
	});
});

describe('EmptyState', () => {
	it('says there is nothing yet and what to do about it', async () => {
		await render(EmptyState, { icon: Bell, title: 'Reminders', count: 0, body: 'Nothing scheduled today. Ask Kala, or add one yourself.', action: { label: 'Add reminder', href: '/reminders/new' } });
		await expect.element(page.getByText('Nothing scheduled today. Ask Kala, or add one yourself.')).toBeInTheDocument();
		await expect.element(page.getByRole('link', { name: /Add reminder/ })).toHaveAttribute('href', '/reminders/new');
		expect(getComputedStyle(page.getByText('0').element()).fontFamily).toContain('Geist Mono');
	});
});

describe('SkeletonRow', () => {
	it('stands in while loading and says what is loading', async () => {
		await render(SkeletonRow, { label: 'loading 4 projects…' });
		const status = page.getByRole('status');
		await expect.element(status).toHaveTextContent('loading 4 projects…');
		await expect.element(status).toHaveAttribute('aria-busy', 'true');
		expect(getComputedStyle(page.getByText('loading 4 projects…').element()).fontFamily).toContain('Geist Mono');
	});
});
