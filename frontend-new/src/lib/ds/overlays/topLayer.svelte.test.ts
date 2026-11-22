import { afterEach, describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import ToastRegion from './ToastRegion.svelte';
import PhoneToastRegion from './PhoneToastRegion.svelte';
import { toasts } from './toasts.svelte';

const tick = () => new Promise((resolve) => setTimeout(resolve, 50));

function openModal(): HTMLDialogElement {
	const dialog = document.createElement('dialog');
	dialog.textContent = 'A sheet';
	document.body.append(dialog);
	dialog.showModal();
	return dialog;
}

afterEach(() => {
	toasts.clear();
	document.querySelectorAll('dialog').forEach((dialog) => dialog.remove());
});

describe('toasts over dialogs and sheets', () => {
	for (const [name, Region] of [
		['ToastRegion', ToastRegion],
		['PhoneToastRegion', PhoneToastRegion],
	] as const) {
		it(`${name} shows above an open modal and can be pressed, then goes back when it closes`, async () => {
			render(Region);
			const dialog = openModal();
			await tick();
			toasts.show({ kind: 'success', title: 'Deploy #2 is live', action: { label: 'Roll back to #1', onclick: () => {} } });
			await expect.element(page.getByText('Deploy #2 is live')).toBeVisible();
			const region = page.getByRole('region', { name: 'Notifications' }).element() as HTMLElement;
			expect(dialog.contains(region)).toBe(true);
			expect(region.matches(':popover-open')).toBe(true);
			const button = page.getByRole('button', { name: 'Roll back to #1' }).element();
			const box = button.getBoundingClientRect();
			expect(document.elementFromPoint(box.x + box.width / 2, box.y + box.height / 2)?.closest('button')).toBe(button);
			dialog.close();
			await tick();
			expect(region.parentElement).toBe(document.body);
			expect(region.matches(':popover-open')).toBe(true);
		});
	}

	it('fits a narrow screen, 16 from each side, wrapping a long detail', async () => {
		await page.viewport(375, 812);
		render(ToastRegion);
		toasts.show({ kind: 'error', title: 'Couldn’t publish deploy #3', detail: 'src/lib/components/checkout/PaymentSummaryWithInstallments.svelte' });
		await expect.element(page.getByRole('alert')).toBeVisible();
		const toast = page.getByRole('alert').element() as HTMLElement;
		const box = toast.getBoundingClientRect();
		expect(box.left).toBe(16);
		expect(box.right).toBe(375 - 16);
		expect(document.documentElement.scrollWidth).toBe(375);
	});
});
