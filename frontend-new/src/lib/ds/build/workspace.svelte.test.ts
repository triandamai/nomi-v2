import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import ResizeHandle from './ResizeHandle.svelte';
import PublishButton from './PublishButton.svelte';
import WorkspaceSheetFixture from './WorkspaceSheetFixture.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

describe('ResizeHandle', () => {
	it('is a focusable separator that moves the split with the keyboard and resets on double-click', async () => {
		const onresize = vi.fn();
		await render(ResizeHandle, { width: 520, available: 1600, onresize });
		const handle = page.getByRole('separator', { name: 'Resize chat' });
		await expect.element(handle).toHaveAttribute('aria-valuenow', '520');
		await expect.element(handle).toHaveAttribute('aria-valuemin', '360');
		await expect.element(handle).toHaveAttribute('aria-valuemax', '720');
		await handle.click();
		await userEvent.keyboard('{ArrowRight}');
		expect(onresize).toHaveBeenLastCalledWith(536);
		await userEvent.dblClick(handle);
		expect(onresize).toHaveBeenLastCalledWith(520);
		expect(handle.element().getBoundingClientRect().width).toBe(9);
	});

	it('turns mint while dragging', async () => {
		const { container } = await render(ResizeHandle, { width: 520, available: 1600, onresize: () => {}, 'data-dragging': true });
		expect(getComputedStyle(container.querySelector('[data-grip]')!).backgroundColor).toBe(tokenBackground('nm-accent'));
	});
});

describe('PublishButton', () => {
	it('is quiet when up to date', async () => {
		await render(PublishButton, { state: 'idle' });
		await expect.element(page.getByRole('button', { name: 'Up to date' })).toBeDisabled();
	});

	it('publishes what is waiting, the count being what will go live', async () => {
		const onpublish = vi.fn();
		await render(PublishButton, { state: 'ready', count: 2, onpublish });
		const button = page.getByRole('button', { name: /Publish/ });
		expect(getComputedStyle(button.element()).backgroundColor).toBe(tokenBackground('nm-accent'));
		await expect.element(button).toHaveTextContent('2');
		await button.click();
		expect(onpublish).toHaveBeenCalledOnce();
	});

	it('shows the time while publishing and offers a retry when it fails', async () => {
		const { rerender } = await render(PublishButton, { state: 'publishing', elapsed: '0:12' });
		await expect.element(page.getByRole('button', { name: 'Publishing… 0:12' })).toHaveAttribute('aria-busy', 'true');
		const onretry = vi.fn();
		await rerender({ state: 'failed', onretry });
		const failed = page.getByRole('button', { name: 'Publish failed · Retry' });
		expect(getComputedStyle(failed.element()).color).toBe(tokenColor('nm-danger'));
		await failed.click();
		expect(onretry).toHaveBeenCalledOnce();
	});
});

describe('WorkspaceSheet', () => {
	it('opens over the preview with a title, its project, expand and close, and closes on Escape', async () => {
		const onclose = vi.fn();
		await render(WorkspaceSheetFixture, { onclose });
		const sheet = page.getByRole('region', { name: 'Domain' });
		await expect.element(sheet).toBeVisible();
		await expect.element(page.getByText('Sheet content')).toBeVisible();
		await expect.element(page.getByText('esc closes · preview keeps running')).toBeVisible();
		await page.getByRole('button', { name: 'Expand' }).click();
		await page.getByText('Sheet content').click();
		await userEvent.keyboard('{Escape}');
		expect(onclose).toHaveBeenCalledOnce();
	});
});
