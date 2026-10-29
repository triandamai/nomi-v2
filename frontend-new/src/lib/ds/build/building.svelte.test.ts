import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import FileList from './FileList.svelte';
import PreviewBuilding from './PreviewBuilding.svelte';
import { tokenColor } from '../test-utils';

// Build → Building: the real step, the real file, the real clock. No percentage, no bar. The
// preview stays empty until there is something real to show.

describe('FileList', () => {
	it('lists each file as written (with its lines), being written, or still to come', async () => {
		const { container } = await render(FileList, {
			files: [
				{ path: 'package.json', change: '+38', state: 'done' },
				{ path: 'src/routes/menu/+page.svelte', state: 'writing' },
				{ path: 'src/routes/order/+page.svelte', state: 'queued' },
			],
			total: 9,
		});
		await expect.element(page.getByText('Files')).toBeVisible();
		await expect.element(page.getByText('1 of 9 written')).toBeVisible();
		expect(page.getByRole('listitem').elements()).toHaveLength(3);
		expect(container.querySelector('svg.lucide-check')!.parentElement!.className).toContain('text-accent');
		expect(getComputedStyle(page.getByText('src/routes/menu/+page.svelte').element()).color).toBe(tokenColor('nm-text'));
		await expect.element(page.getByText('writing', { exact: true })).toBeVisible();
		const queued = page.getByText('src/routes/order/+page.svelte').element().closest('li')!;
		expect(getComputedStyle(queued).opacity).toBe('0.45');
	});
});

describe('PreviewBuilding', () => {
	it('stays empty while building, saying where the preview will appear and the real steps', async () => {
		const { container } = await render(PreviewBuilding, {
			steps: [
				{ label: 'Set up project and database', state: 'done' },
				{ label: 'Menu page', state: 'running' },
				{ label: 'Pre-order form', state: 'waiting' },
			],
		});
		await expect.element(page.getByRole('heading', { name: 'Building your app' })).toBeVisible();
		await expect.element(page.getByText('The preview opens here after the first deploy')).toBeVisible();
		expect(getComputedStyle(page.getByText('Pre-order form').element()).color).toBe(tokenColor('nm-text-3'));
		expect(container.querySelector('svg.lucide-circle-dashed')).not.toBeNull();
		expect(container.firstElementChild!.getBoundingClientRect().height).toBe(300);
	});
});
