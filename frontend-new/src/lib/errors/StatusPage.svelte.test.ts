import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import StatusPage from './StatusPage.svelte';

describe('StatusPage', () => {
	it('a 404 says the page wandered off, with the crew looking for it and a way home', async () => {
		render(StatusPage, { status: 404 });
		await expect.element(page.getByRole('heading', { level: 1 })).toHaveTextContent('This page wandered off');
		expect(document.querySelector('[data-scene="lost"]')).not.toBeNull();
		await expect.element(page.getByRole('link', { name: 'Back to Nomi' })).toHaveAttribute('href', '/');
		expect(page.getByRole('button', { name: 'Try again' }).query()).toBeNull();
	});

	it('an error says something broke, quotes its reason and offers Try again', async () => {
		render(StatusPage, { status: 503, message: 'The backend is restarting' });
		await expect.element(page.getByRole('heading', { level: 1 })).toHaveTextContent('Something broke on our side');
		expect(document.querySelector('[data-scene="broken"]')).not.toBeNull();
		await expect.element(page.getByText('503')).toBeVisible();
		await expect.element(page.getByText('The backend is restarting')).toBeVisible();
		await expect.element(page.getByRole('button', { name: 'Try again' })).toBeVisible();
	});

	it('leaves SvelteKit’s own placeholder and the generic line out', async () => {
		render(StatusPage, { status: 500, message: 'Internal Error' });
		await expect.element(page.getByRole('heading', { level: 1 })).toBeVisible();
		expect(page.getByText('Internal Error').query()).toBeNull();
	});
});
