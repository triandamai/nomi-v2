import { describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import CrewSection from './CrewSection.svelte';

describe('Crew behaviour section', () => {
	it('shows hand-offs, limits, memory, connections, approval editing and background work', async () => {
		await render(CrewSection);
		await expect.element(page.getByText('Nomi handed this to Dana')).toBeInTheDocument();
		await expect.element(page.getByText("I can't cancel subscriptions for you")).toBeInTheDocument();
		await expect.element(page.getByText('Connect Google Calendar')).toBeInTheDocument();
		await expect.element(page.getByRole('textbox', { name: "Editing Tara's email" })).toBeInTheDocument();
		await expect.element(page.getByText('Supervisor stopped 3 tasks')).toBeInTheDocument();
		await expect.element(page.getByRole('heading', { name: "You've been signed out" })).toBeInTheDocument();
	});
});
