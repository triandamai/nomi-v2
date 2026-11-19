import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { AlarmClock, Inbox } from '@lucide/svelte';
import PhoneAskCard from './PhoneAskCard.svelte';
import PhoneTodayList from './PhoneTodayList.svelte';
import PhoneCrewRow from './PhoneCrewRow.svelte';
import { tokenBackground } from '../test-utils';

// Home on a phone (Flow / Mobile — Home, Inbox & You, step 1 dl13j): the ask card, Today as rows,
// and the crew's faces.

afterEach(() => {
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();

describe('PhoneAskCard', () => {
	it('sends what’s typed, filled only once there is something, and opens who it goes to and the model', async () => {
		await page.viewport(390, 844);
		const onsend = vi.fn();
		const onto = vi.fn();
		const onmodel = vi.fn();
		await render(PhoneAskCard, { value: '', placeholder: 'Ask Nomi', to: { agent: 'nomi', name: 'Nomi' }, model: 'deepseek-flash', onsend, onto, onmodel });
		const send = page.getByRole('button', { name: 'Send', exact: true });
		await expect.element(send).toBeDisabled();
		await userEvent.type(page.getByPlaceholder('Ask Nomi'), 'Plan my weekend');
		await expect.element(send).toBeEnabled();
		expect(getComputedStyle(send.element()).backgroundColor).toBe(tokenBackground('nm-accent'));
		await send.click();
		expect(onsend).toHaveBeenCalledWith('Plan my weekend');
		await page.getByRole('button', { name: 'Send to Nomi, change' }).click();
		expect(onto).toHaveBeenCalledOnce();
		await page.getByRole('button', { name: /deepseek-flash/ }).click();
		expect(onmodel).toHaveBeenCalledOnce();
	});
});

describe('PhoneTodayList', () => {
	it('is 64-high rows of links, each its title, its line and its count', async () => {
		await page.viewport(390, 844);
		await render(PhoneTodayList, {
			label: 'Today',
			rows: [
				{ icon: Inbox, title: 'While you were out', detail: 'A deploy failed · 2 updates', count: 3, href: '/chats' },
				{ icon: AlarmClock, title: 'Reminders today', detail: 'Stretch 16:00', count: 2, href: '/inbox' },
			],
		});
		const nav = page.getByRole('navigation', { name: 'Today' }).element();
		const links = [...nav.querySelectorAll('a')];
		expect(links.map((link) => [link.getAttribute('href'), Math.round(box(link).height)])).toEqual([
			['/chats', 64],
			['/inbox', 64],
		]);
		await expect.element(page.getByRole('link', { name: /While you were out.*A deploy failed.*3/ })).toBeVisible();
	});
});

describe('PhoneCrewRow', () => {
	it('shows up to five members, each a link to the crew', async () => {
		await page.viewport(390, 844);
		const names = ['Nomi', 'Dana', 'Kala', 'Maya', 'Tara', 'Raga'];
		await render(PhoneCrewRow, { label: 'Your crew', href: '/settings/crew', members: names.map((name) => ({ agent: name.toLowerCase() as 'nomi', name })) });
		const links = [...page.getByRole('navigation', { name: 'Your crew' }).element().querySelectorAll('a')];
		expect(links.map((link) => [link.textContent?.trim(), link.getAttribute('href')])).toEqual(names.slice(0, 5).map((name) => [name, '/settings/crew']));
	});
});
