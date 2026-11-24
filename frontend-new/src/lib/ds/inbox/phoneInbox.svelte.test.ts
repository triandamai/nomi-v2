import { afterEach, describe, expect, it } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page } from 'vitest/browser';
import { createRawSnippet } from 'svelte';
import PhoneInboxCard from './PhoneInboxCard.svelte';
import PhoneTimelineRow from './PhoneTimelineRow.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// The Inbox on a phone (Flow / Mobile — Home, Inbox & You, step 2 fpYK0): what needs the person as
// cards, the day as a timeline.

afterEach(() => {
	document.body.innerHTML = '';
});

describe('PhoneInboxCard', () => {
	it('is a card named by its title: the kind and when, the dot while unread, the line and its actions', async () => {
		await page.viewport(390, 844);
		await render(PhoneInboxCard, {
			kind: 'account',
			when: '18 Oct',
			title: 'Trian invited you',
			body: 'Work on its 4 projects together.',
			unread: true,
			meta: 'expires in 4 days',
			actions: createRawSnippet(() => ({ render: () => '<button type="button">Open invite</button>' })),
		});
		const card = page.getByRole('article', { name: 'Trian invited you' });
		await expect.element(card).toBeVisible();
		const style = getComputedStyle(card.element());
		expect([style.borderRadius, style.paddingTop, style.backgroundColor]).toEqual(['16px', '14px', tokenBackground('nm-surface')]);
		expect(card.element().querySelector('[aria-hidden="true"]')).not.toBeNull();
		await expect.element(page.getByRole('heading', { name: 'Trian invited you' })).toBeVisible();
		await expect.element(page.getByRole('button', { name: 'Open invite' })).toBeVisible();
		await expect.element(page.getByText('expires in 4 days')).toBeVisible();
	});
});

describe('PhoneTimelineRow', () => {
	it('dims a done row’s title and shows where it stands in the accent', async () => {
		await page.viewport(390, 844);
		await render(PhoneTimelineRow, { time: '06:30', title: 'Gym', meta: 'reminder · Mon Wed Fri', state: 'Done 06:42', done: true });
		expect(getComputedStyle(page.getByText('Gym').element()).color).toBe(tokenColor('nm-text-2'));
		expect(getComputedStyle(page.getByText('Done 06:42').element()).color).toBe(tokenColor('nm-accent'));
	});

	it('keeps one still to come in text, where it stands quiet', async () => {
		await page.viewport(390, 844);
		await render(PhoneTimelineRow, { time: '16:00', title: 'Stretch', state: 'in 7h 46m' });
		expect(getComputedStyle(page.getByText('Stretch').element()).color).toBe(tokenColor('nm-text'));
		expect(getComputedStyle(page.getByText('in 7h 46m').element()).color).toBe(tokenColor('nm-text-3'));
	});
});
