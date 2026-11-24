import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { createRawSnippet } from 'svelte';
import { Lock, Mail, Trash2 } from '@lucide/svelte';
import PhoneSettingsGroup from './PhoneSettingsGroup.svelte';
import PhoneFormScreen from './PhoneFormScreen.svelte';
import PhoneBackBar from '../chrome/PhoneBackBar.svelte';
import { tokenColor } from '../test-utils';

// Flow / Mobile — You & Settings (EtSi2): the rows every settings screen is made of, the back bar,
// and the screen typing gets.

afterEach(() => {
	document.body.innerHTML = '';
});

const box = (element: Element) => element.getBoundingClientRect();

describe('PhoneSettingsGroup', () => {
	it('shows what each row has set, and opens what changes it', async () => {
		await page.viewport(390, 844);
		const open = vi.fn();
		await render(PhoneSettingsGroup, {
			label: 'Account & sign-in',
			rows: [
				{ icon: Mail, label: 'Email', meta: 'si•••@nomi.test', mono: true, onclick: open },
				{ icon: Lock, label: 'Password', meta: 'Change', href: '/settings/general#password' },
			],
		});
		const email = page.getByRole('button', { name: /Email/ });
		expect(Math.round(box(email.element()).height)).toBe(52);
		expect(getComputedStyle(email.element().querySelector('.font-mono')!).fontFamily).toMatch(/Mono/);
		await email.click();
		expect(open).toHaveBeenCalledOnce();
		await expect.element(page.getByRole('link', { name: /Password.*Change/ })).toHaveAttribute('href', '/settings/general#password');
		await expect.element(page.getByRole('region', { name: 'Account & sign-in' })).toBeVisible();
	});

	it('holds a switch the row names, saved as it flips', async () => {
		const flip = vi.fn();
		await render(PhoneSettingsGroup, { label: 'Chat', rows: [{ label: 'Enter sends', sub: 'Shift+Enter adds a new line', toggle: { checked: true, onchange: flip } }] });
		const toggle = page.getByRole('switch', { name: 'Enter sends' });
		await expect.element(toggle).toBeChecked();
		await toggle.click();
		expect(flip).toHaveBeenCalledWith(false);
	});

	it('ticks a sheet’s chosen choice, and draws danger without a chevron', async () => {
		await render(PhoneSettingsGroup, {
			rows: [
				{ label: 'Bahasa Indonesia', chosen: true, onclick: () => {} },
				{ icon: Trash2, label: 'Delete account', danger: true, onclick: () => {} },
			],
		});
		await expect.element(page.getByRole('button', { name: 'Bahasa Indonesia' })).toHaveAttribute('aria-pressed', 'true');
		const danger = page.getByRole('button', { name: 'Delete account' }).element();
		expect(getComputedStyle(danger.querySelector('.text-danger')!).color).toBe(tokenColor('nm-danger'));
		expect(danger.querySelectorAll('svg')).toHaveLength(1);
	});
});

describe('PhoneBackBar', () => {
	it('is 48 high, its way back a 44 target, as a link or a button', async () => {
		await page.viewport(390, 844);
		await render(PhoneBackBar, { label: 'You', href: '/settings' });
		const back = page.getByRole('link', { name: 'You' });
		await expect.element(back).toHaveAttribute('href', '/settings');
		expect(Math.round(box(back.element()).height)).toBe(44);
		expect(Math.round(box(back.element().parentElement!).height)).toBe(48);
		document.body.innerHTML = '';
		const onback = vi.fn();
		await render(PhoneBackBar, { label: 'General', onback });
		await page.getByRole('button', { name: 'General' }).click();
		expect(onback).toHaveBeenCalledOnce();
	});
});

describe('PhoneFormScreen', () => {
	it('names the screen, submits from the keyboard, and keeps its fields at 16pt', async () => {
		await page.viewport(390, 844);
		const submit = vi.fn();
		await render(PhoneFormScreen, {
			backLabel: 'General',
			onback: () => {},
			title: 'Change your email',
			sub: 'We’ll send a code to the new address.',
			onsubmit: submit,
			children: createRawSnippet(() => ({ render: () => '<input aria-label="New email" type="email" />' })),
			action: createRawSnippet(() => ({ render: () => '<button type="submit">Send code</button>' })),
		});
		await expect.element(page.getByRole('heading', { level: 1, name: 'Change your email' })).toBeVisible();
		const field = page.getByRole('textbox', { name: 'New email' });
		expect(getComputedStyle(field.element()).fontSize).toBe('16px');
		await userEvent.type(field, 'siti@kopikita.id{Enter}');
		expect(submit).toHaveBeenCalledOnce();
	});
});
