import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import SandboxWait from './SandboxWait.svelte';

// The preview pane while the sandbox isn't serving the app yet (Code, terminal & phone preview uR6dq):
// what it's waiting for, and what it's doing now when it says.

describe('SandboxWait', () => {
	it('says the preview waits for the first build, and starts nothing', async () => {
		await render(SandboxWait, { phase: 'building', log: '', error: null, onretry: () => {} });
		await expect.element(page.getByText('The preview starts when Koda finishes the first build.')).toBeInTheDocument();
		expect(page.getByRole('button').elements()).toHaveLength(0);
	});

	it('shows what the sandbox is doing while it starts the app', async () => {
		await render(SandboxWait, { phase: 'starting', log: 'npm install\nadded 212 packages\n', error: null, onretry: () => {} });
		await expect.element(page.getByText('Starting the app')).toBeInTheDocument();
		await expect.element(page.getByText('added 212 packages')).toBeInTheDocument();
	});

	it('shows no progress line before the sandbox printed anything', async () => {
		const { container } = await render(SandboxWait, { phase: 'waking', log: '', error: null, onretry: () => {} });
		await expect.element(page.getByText('Waking the sandbox')).toBeInTheDocument();
		expect(container.querySelector('[data-progress]')).toBeNull();
	});

	it('says why it failed and offers to start it again', async () => {
		const onretry = vi.fn();
		await render(SandboxWait, { phase: 'failed', log: 'boom', error: 'npm install failed', onretry });
		await expect.element(page.getByText('npm install failed')).toBeInTheDocument();
		await page.getByRole('button', { name: 'Start again' }).click();
		expect(onretry).toHaveBeenCalledOnce();
	});
});
