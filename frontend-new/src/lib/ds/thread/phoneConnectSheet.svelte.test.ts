import { describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import PhoneConnectSheet from './PhoneConnectSheet.svelte';

const resume = '11111111-2222-3333-4444-555555555555';

// Flow / Mobile Chat — Crew, memory & connections, step 5 (SYohQ): the app and who needs it, what
// access means, Connect with Google (46, accent) and answering yourself (46, outlined).
describe('PhoneConnectSheet', () => {
	it('starts Google’s sign-in for the services asked, coming back to this chat', async () => {
		await render(PhoneConnectSheet, { open: true, agent: 'tara', agentName: 'Tara', service: 'Calendar', provider: 'google', services: ['calendar'], resume, onanswer: () => {} });
		await expect.element(page.getByRole('dialog', { name: 'Connect Google Calendar' })).toBeVisible();
		await expect.element(page.getByText('Tara needs it to answer you')).toBeVisible();
		await expect.element(page.getByText('Nothing is created or changed without asking')).toBeVisible();
		const connect = page.getByRole('button', { name: 'Connect with Google' });
		expect(connect.element().getBoundingClientRect().height).toBe(46);
		const form = connect.element().closest('form')!;
		expect(form.getAttribute('action')).toBe('/settings/connections?/google');
		expect(new FormData(form).getAll('service')).toEqual(['calendar']);
		expect(new FormData(form).get('resume')).toBe(resume);
	});

	it('lets the person answer instead, and sends any other app to Connections', async () => {
		const onanswer = vi.fn();
		await render(PhoneConnectSheet, { open: true, agent: 'raga', agentName: 'Raga', service: 'Hevy', provider: 'other', services: [], resume, onanswer });
		await expect.element(page.getByRole('link', { name: 'Open Connections' })).toHaveAttribute('href', '/settings/connections');
		await page.getByRole('button', { name: 'I’ll tell Raga myself' }).click();
		expect(onanswer).toHaveBeenCalledOnce();
		await expect.element(page.getByRole('dialog')).not.toBeInTheDocument();
	});
});
