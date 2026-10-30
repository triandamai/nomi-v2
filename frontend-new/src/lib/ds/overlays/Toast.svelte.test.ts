import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import Toast from './Toast.svelte';
import ToastRegion from './ToastRegion.svelte';
import { toasts } from './toasts.svelte';
import { tokenColor } from '../test-utils';

afterEach(() => {
	toasts.clear();
	vi.useRealTimers();
});

describe('Toast', () => {
	it('says what happened, the detail in mono, with one action and a close', async () => {
		const onaction = vi.fn();
		const ondismiss = vi.fn();
		const { container } = await render(Toast, {
			kind: 'success',
			title: 'Deployed to salah-daily-tracker-0ab8f8.trian.space',
			detail: 'deploy #15 · built in 38s',
			action: { label: 'Visit', onclick: onaction },
			ondismiss,
		});
		const toast = page.getByRole('status');
		await expect.element(toast).toHaveTextContent('Deployed to salah-daily-tracker-0ab8f8.trian.space');
		expect(getComputedStyle(toast.element()).boxShadow).toContain('0px 8px 24px');
		expect(getComputedStyle(container.querySelector('svg.lucide-check')!).stroke).toBe(tokenColor('nm-accent'));
		expect(getComputedStyle(page.getByText('deploy #15 · built in 38s').element()).fontFamily).toContain('Geist Mono');
		await page.getByRole('button', { name: 'Visit' }).click();
		await page.getByRole('button', { name: 'Dismiss' }).click();
		expect(onaction).toHaveBeenCalledOnce();
		expect(ondismiss).toHaveBeenCalledOnce();
	});

	it('interrupts for an error', async () => {
		const { container } = await render(Toast, { kind: 'error', title: 'Deploy failed: build error in src/lib/pdf.ts', detail: 'Koda can try a fix, or open the log.' });
		await expect.element(page.getByRole('alert')).toHaveTextContent('Deploy failed');
		expect(getComputedStyle(container.querySelector('svg.lucide-circle-alert')!).stroke).toBe(tokenColor('nm-danger'));
	});
});

describe('toasts', () => {
	it('shows toasts in a region and lets them go after a while, except errors', async () => {
		vi.useFakeTimers();
		await render(ToastRegion);
		toasts.show({ kind: 'info', title: 'Repo exported: salah-daily-tracker.zip', detail: '23 files · 184 KB' });
		toasts.show({ kind: 'error', title: 'Deploy failed' });
		await expect.element(page.getByText('Repo exported: salah-daily-tracker.zip')).toBeInTheDocument();
		vi.advanceTimersByTime(6100);
		await expect.element(page.getByText('Repo exported: salah-daily-tracker.zip')).not.toBeInTheDocument();
		await expect.element(page.getByText('Deploy failed')).toBeInTheDocument();
	});

	it('keeps an undo toast for its own time, and runs the action once', async () => {
		vi.useFakeTimers();
		const undo = vi.fn();
		await render(ToastRegion);
		toasts.show({ kind: 'info', title: 'Reply deleted', action: { label: 'Undo', onclick: undo }, duration: 10_000 });
		vi.advanceTimersByTime(9000);
		await page.getByRole('button', { name: 'Undo' }).click();
		expect(undo).toHaveBeenCalledOnce();
		await expect.element(page.getByText('Reply deleted')).not.toBeInTheDocument();
	});
});
