import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import MessageUser from '../conversation/MessageUser.svelte';
import ReplyText from './ReplyText.svelte';
import Banner from './Banner.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Chat responses → States & errors: a message says whether it went, a reply says whether it's
// still coming, stopped or cut off, and banners above the thread say what's wrong around it.

describe('MessageUser states', () => {
	it('shows it is sending, a little faded', async () => {
		const { container } = await render(MessageUser, { text: 'Kirim bukti transfer ke Bu Sri ya', status: 'sending' });
		await expect.element(page.getByText('sending…')).toBeVisible();
		expect(container.querySelector('svg.lucide-loader-circle')).not.toBeNull();
		expect(getComputedStyle(page.getByText('Kirim bukti transfer ke Bu Sri ya').element()).opacity).toBe('0.85');
	});

	it('waits in the outbox while offline, and can be cancelled', async () => {
		const oncancel = vi.fn();
		await render(MessageUser, { text: 'Kirim', status: 'queued', oncancel });
		await expect.element(page.getByText("queued · sends when you're back online")).toBeVisible();
		await page.getByRole('button', { name: 'Cancel' }).click();
		expect(oncancel).toHaveBeenCalledOnce();
	});

	it('says plainly it was not sent, with retry and edit', async () => {
		const onretry = vi.fn();
		const onedit = vi.fn();
		await render(MessageUser, { text: 'Kirim', status: 'failed', onretry, onedit });
		expect(getComputedStyle(page.getByText('not sent').element()).color).toBe(tokenColor('nm-danger'));
		expect(getComputedStyle(page.getByText('Kirim').element()).outlineColor).toBe(tokenColor('nm-danger-border'));
		await page.getByRole('button', { name: 'Retry' }).click();
		await page.getByRole('button', { name: 'Edit' }).click();
		expect(onretry).toHaveBeenCalledOnce();
		expect(onedit).toHaveBeenCalledOnce();
	});

	it('shows nothing extra once sent', async () => {
		await render(MessageUser, { text: 'Kirim' });
		expect(page.getByRole('button').elements()).toHaveLength(0);
		expect(getComputedStyle(page.getByText('Kirim').element()).opacity).toBe('1');
	});
});

describe('ReplyText', () => {
	const text = "Food is October's biggest jump: Rp 2,1 jt against Rp 1,4 jt in September. Most of the difference is GoFood after";

	it('streams with a mint caret, busy until done', async () => {
		const { container } = await render(ReplyText, { text, status: 'streaming' });
		const caret = container.querySelector('[data-caret]')!;
		expect(getComputedStyle(caret).backgroundColor).toBe(tokenBackground('nm-accent'));
		expect(container.querySelector('[aria-busy="true"]')).not.toBeNull();
	});

	it('marks where you stopped it, how long it ran, and continues', async () => {
		const oncontinue = vi.fn();
		const { container } = await render(ReplyText, { text, status: 'stopped', elapsed: '00:03', oncontinue });
		expect(container.querySelector('[data-caret]')).toBeNull();
		expect(getComputedStyle(page.getByText('stopped by you · 00:03').element()).fontFamily).toContain('Geist Mono');
		await page.getByRole('button', { name: 'Continue' }).click();
		expect(oncontinue).toHaveBeenCalledOnce();
	});

	it('says why it was cut off, with start over and continue', async () => {
		const onrestart = vi.fn();
		const oncontinue = vi.fn();
		await render(ReplyText, { text, status: 'interrupted', reason: 'Reply cut off after 18s. DeepSeek closed the stream.', onrestart, oncontinue });
		const notice = page.getByText('Reply cut off after 18s. DeepSeek closed the stream.').element().parentElement!;
		expect(getComputedStyle(notice).backgroundColor).toBe(tokenBackground('nm-warning-bg'));
		await page.getByRole('button', { name: 'Start over' }).click();
		await page.getByRole('button', { name: 'Continue' }).click();
		expect(onrestart).toHaveBeenCalledOnce();
		expect(oncontinue).toHaveBeenCalledOnce();
	});
});

describe('Banner', () => {
	it('warns you are offline and offers a retry', async () => {
		const onclick = vi.fn();
		const { container } = await render(Banner, { kind: 'offline', text: "You're offline. New messages wait in the outbox and send when you reconnect.", action: { label: 'Retry now', onclick } });
		const banner = page.getByRole('status');
		expect(getComputedStyle(banner.element()).backgroundColor).toBe(tokenBackground('nm-warning-bg'));
		expect(container.querySelector('svg.lucide-wifi-off')).not.toBeNull();
		await page.getByRole('button', { name: 'Retry now' }).click();
		expect(onclick).toHaveBeenCalledOnce();
	});

	it('says you are back online in mint', async () => {
		await render(Banner, { kind: 'online', text: 'Back online. 1 queued message sent.' });
		expect(getComputedStyle(page.getByRole('status').element()).backgroundColor).toBe(tokenBackground('nm-accent-bg'));
	});

	it('points to the provider’s status page and to usage, on a bordered surface', async () => {
		const { container } = await render(Banner, { kind: 'slow', text: 'DeepSeek is slow right now (replies ~14s). Your chats still work.', action: { label: 'DeepSeek status', href: 'https://status.deepseek.com', external: true } });
		expect(container.querySelector('svg.lucide-activity')).not.toBeNull();
		expect(getComputedStyle(page.getByRole('status').element()).outlineColor).toBe(tokenColor('nm-border'));
		await expect.element(page.getByRole('link', { name: 'DeepSeek status' })).toHaveAttribute('target', '_blank');
	});

	it('has a key and a quota kind', async () => {
		const { container } = await render(Banner, { kind: 'expiring', text: 'Your DeepSeek key expires in 3 days.' });
		expect(container.querySelector('svg.lucide-key-round')).not.toBeNull();
		const quota = await render(Banner, { kind: 'quota', text: '82% used' });
		expect(quota.container.querySelector('svg.lucide-gauge')).not.toBeNull();
	});
});
