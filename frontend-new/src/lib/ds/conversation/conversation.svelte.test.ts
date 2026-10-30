import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { ArrowUpRight, GitBranch, Hourglass, RotateCcw } from '@lucide/svelte';
import MessageUser from './MessageUser.svelte';
import MessageAgent from './MessageAgent.svelte';
import ErrorCallout from './ErrorCallout.svelte';
import ActivityBlock from './ActivityBlock.svelte';
import FileRow from './FileRow.svelte';
import LiveStep from './LiveStep.svelte';
import EventRow from './EventRow.svelte';
import KVRow from './KVRow.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Agent activity is legible, never theatrical: real tool, real target, real time.

describe('MessageUser', () => {
	it('sits on the right in a surface-2 bubble', async () => {
		await render(MessageUser, { text: 'hey' });
		const bubble = page.getByText('hey').element();
		const style = getComputedStyle(bubble);
		expect(style.backgroundColor).toBe(tokenBackground('nm-surface-2'));
		expect(style.borderTopLeftRadius).toBe('10px');
		expect(style.fontSize).toBe('15px');
		const row = bubble.parentElement!.getBoundingClientRect();
		expect(Math.round(bubble.getBoundingClientRect().right)).toBe(Math.round(row.right));
		expect(bubble.getBoundingClientRect().left).toBeGreaterThan(row.left);
	});
});

describe('MessageAgent', () => {
	it('says who answered and when, then the answer in 15px text-2', async () => {
		await render(MessageAgent, { agent: 'nomi', name: 'Nomi', meta: '14:02 · deepseek-flash', text: 'Hello there.' });
		await expect.element(page.getByText('14:02 · deepseek-flash')).toBeInTheDocument();
		const body = getComputedStyle(page.getByText('Hello there.').element());
		expect(body.fontSize).toBe('15px');
		expect(body.color).toBe(tokenColor('nm-text-2'));
		expect(body.lineHeight).toBe('24px');
	});
});

describe('ErrorCallout', () => {
	it('names the cause, quotes the provider in mono and offers the fix', async () => {
		const onretry = vi.fn();
		await render(ErrorCallout, {
			title: 'Provider error',
			code: '402 insufficient_balance',
			requestId: 'req_9d41e0b2',
			details: [
				{ key: 'reason', value: 'Insufficient Balance', strong: true },
				{ key: 'provider', value: 'api.deepseek.com' },
			],
			actions: [
				{ label: 'Retry', icon: RotateCcw, onclick: onretry },
				{ label: 'Add credits on DeepSeek', icon: ArrowUpRight, href: 'https://platform.deepseek.com', external: true },
			],
		});
		const title = page.getByText('Provider error').element();
		expect(getComputedStyle(title).color).toBe(tokenColor('nm-danger'));
		expect(getComputedStyle(title.closest('header')!).backgroundColor).toBe(tokenBackground('nm-danger-bg'));
		for (const machine of ['402 insufficient_balance', 'req_9d41e0b2', 'Insufficient Balance', 'api.deepseek.com']) {
			expect(getComputedStyle(page.getByText(machine).element()).fontFamily, machine).toContain('Geist Mono');
		}
		const retry = page.getByRole('button', { name: 'Retry' });
		// The first action is the inverse one, its icon after the label.
		expect(getComputedStyle(retry.element()).backgroundColor).toBe(tokenBackground('nm-text'));
		expect(retry.element().lastElementChild?.querySelector('svg.lucide-rotate-ccw')).not.toBeNull();
		await retry.click();
		expect(onretry).toHaveBeenCalledOnce();
		const fix = page.getByRole('link', { name: /Add credits on DeepSeek/ });
		await expect.element(fix).toHaveAttribute('href', 'https://platform.deepseek.com');
		// The fix is a secondary button, next to the inverse Retry.
		expect(getComputedStyle(fix.element()).borderTopColor).toBe(tokenColor('nm-border-strong'));
		await expect.element(fix).toHaveAttribute('target', '_blank');
	});

	it('counts down to its own retry, which you can still start now', async () => {
		const onclick = vi.fn();
		await render(ErrorCallout, { title: 'Rate limited', actions: [{ label: 'Retrying in 0:24', icon: Hourglass, onclick }] });
		await page.getByRole('button', { name: 'Retrying in 0:24' }).click();
		expect(onclick).toHaveBeenCalledOnce();
	});
});

describe('ActivityBlock and FileRow', () => {
	it('sums up what was written, file by file, in mono', async () => {
		const { container } = await render(ActivityBlock, {
			title: 'Wrote 3 files',
			total: '+424',
			files: [
				{ path: 'src/routes/+page.svelte', change: '+214' },
				{ path: 'src/lib/prayer-times.ts', change: '+162' },
				{ path: 'src/lib/storage.ts', change: '+48' },
			],
		});
		expect(container.querySelector('svg.lucide-check')).not.toBeNull();
		expect(getComputedStyle(container.querySelector('svg.lucide-check')!).stroke).toBe(tokenColor('nm-accent'));
		expect(page.getByRole('listitem').elements()).toHaveLength(3);
		expect(getComputedStyle(page.getByText('src/lib/storage.ts').element()).fontFamily).toContain('Geist Mono');
		await expect.element(page.getByText('+424')).toBeInTheDocument();
	});

	it('keeps a long path to one line', async () => {
		await render(FileRow, { path: 'src/routes/(app)/projects/[projectId]/settings/domain/+page.svelte', change: '+3' });
		expect(getComputedStyle(page.getByText(/settings\/domain/).element()).textOverflow).toBe('ellipsis');
	});
});

describe('LiveStep', () => {
	it('shows the real step and the time so far, with a way to stop', async () => {
		const onstop = vi.fn();
		const { container } = await render(LiveStep, { step: 'Writing src/routes/menu/+page.svelte', elapsed: '01:12', stopLabel: 'Stop', onstop });
		expect(container.querySelector('svg.lucide-loader-circle')).not.toBeNull();
		await expect.element(page.getByRole('status')).toHaveTextContent('Writing src/routes/menu/+page.svelte');
		await expect.element(page.getByText('01:12')).toBeInTheDocument();
		await page.getByRole('button', { name: 'Stop' }).click();
		expect(onstop).toHaveBeenCalledOnce();
	});
});

describe('EventRow and KVRow', () => {
	it('logs an event in mono with its time', async () => {
		await render(EventRow, { kind: 'deploy', text: 'deploy #14 · main@a3f9c21 · built in 41s', time: '15:38' });
		const text = page.getByText('deploy #14 · main@a3f9c21 · built in 41s').element();
		expect(getComputedStyle(text).fontFamily).toContain('Geist Mono');
		expect(getComputedStyle(text.parentElement!).backgroundColor).toBe(tokenBackground('nm-surface'));
	});

	it('pairs a plain key with a machine value', async () => {
		await render(KVRow, { icon: GitBranch, label: 'Code', value: '23 files · main@a3f9c21' });
		expect(getComputedStyle(page.getByText('Code').element()).fontFamily).not.toContain('Geist Mono');
		expect(getComputedStyle(page.getByText('23 files · main@a3f9c21').element()).fontFamily).toContain('Geist Mono');
	});
});
