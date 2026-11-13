import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { Mail } from '@lucide/svelte';
import { tokenBackground, tokenColor } from '../test-utils';
import AuthButton from './AuthButton.svelte';
import AuthField from './AuthField.svelte';
import AuthLayoutFixture from './AuthLayoutFixture.svelte';
import AuthLink from './AuthLink.svelte';
import AuthNote from './AuthNote.svelte';
import GoogleButton from './GoogleButton.svelte';
import LanguageSwitch from './LanguageSwitch.svelte';
import MailAppButton from './MailAppButton.svelte';
import OrDivider from './OrDivider.svelte';

// Auth / Sign in or create (vDaAQ) and Enter code (pUDXz): the brand panel beside the form on a
// desktop, the form alone on a phone, and the parts the form is made of.

afterEach(async () => {
	await page.viewport(1512, 862);
});

const box = (element: Element) => element.getBoundingClientRect();

describe('AuthLayout', () => {
	it('puts the brand panel (760) beside a 380 form column on a 1512 × 862 screen', async () => {
		await page.viewport(1512, 862);
		await render(AuthLayoutFixture);
		const brand = page.getByRole('complementary').element();
		expect(box(brand)).toMatchObject({ x: 16, y: 16, width: 760, height: 830 });
		expect(getComputedStyle(brand).backgroundColor).toBe(tokenBackground('crew-nomi-bg'));
		await expect.element(page.getByText('A small crew that works for you.')).toBeVisible();
		for (const line of ['Free · 1M tokens a month', 'No card needed', 'Export everything any time']) await expect.element(page.getByText(line)).toBeVisible();
		await expect.element(page.getByText('salah-daily-tracker is live')).toBeVisible();
		expect(getComputedStyle(page.getByText('A small crew that works for you.').element()).fontSize).toBe('40px');

		const main = page.getByRole('main').element();
		expect(box(main)).toMatchObject({ x: 792, width: 704 });
		const column = page.getByRole('heading', { name: 'Sign in to Nomi' }).element().parentElement!;
		expect(box(column).width).toBe(380);
		expect(box(column).x).toBe(792 + 162);
		await expect.element(page.getByRole('button', { name: 'Language here' })).toBeVisible();
	});

	it('greets as Nomi above the form, and can leave the greeting out', async () => {
		await render(AuthLayoutFixture);
		const greeting = page.getByText('Hi, I’m Nomi. Let’s get you in.');
		await expect.element(greeting).toBeVisible();
		expect(getComputedStyle(greeting.element()).backgroundColor).toBe(tokenBackground('nm-surface-2'));
		document.body.innerHTML = '';
		await render(AuthLayoutFixture, { greeting: false });
		expect(page.getByText('Hi, I’m Nomi. Let’s get you in.').elements()).toHaveLength(0);
	});

	it('closes with the terms and the honest status line: the version in mono', async () => {
		await render(AuthLayoutFixture);
		await expect.element(page.getByText('By continuing you agree to the Terms and the Privacy Policy.', { exact: false })).toBeVisible();
		const status = page.getByText('all systems normal · v1.2.3 · © 2026 Nomi');
		await expect.element(status).toBeVisible();
		expect(getComputedStyle(status.element()).fontFamily).toContain('Geist Mono');
		const dot = status.element().previousElementSibling!;
		expect(getComputedStyle(dot).backgroundColor).toBe(tokenBackground('nm-accent'));
	});

	it('says so when Nomi is down for maintenance', async () => {
		await render(AuthLayoutFixture, { healthy: false });
		const status = page.getByText('under maintenance · v1.2.3 · © 2026 Nomi');
		await expect.element(status).toBeVisible();
		expect(getComputedStyle(status.element().previousElementSibling!).backgroundColor).toBe(tokenBackground('nm-warning'));
	});

	it('drops the brand panel on a phone and gives the form the whole width', async () => {
		await page.viewport(390, 844);
		await render(AuthLayoutFixture);
		expect(page.getByRole('complementary').elements()).toHaveLength(0);
		expect(getComputedStyle(document.querySelector('aside')!).display).toBe('none');
		const column = page.getByRole('heading', { name: 'Sign in to Nomi' }).element().parentElement!;
		expect(box(column)).toMatchObject({ x: 16, width: 358 });
	});
});

describe('LanguageSwitch', () => {
	const options = [
		{ value: 'en', label: 'English' },
		{ value: 'id', label: 'Bahasa Indonesia' },
	];

	it('shows the language in use, 28 high, and switches with the platform picker', async () => {
		const onchange = vi.fn();
		await render(LanguageSwitch, { value: 'en', options, onchange });
		const select = page.getByRole('combobox', { name: 'Language' });
		await expect.element(select).toHaveValue('en');
		const pill = select.element().parentElement!;
		expect(getComputedStyle(pill).height).toBe('28px');
		expect(pill.textContent).toContain('English');
		await select.selectOptions('id');
		expect(onchange).toHaveBeenCalledWith('id');
	});

	it('is a 44px target on a phone', async () => {
		await page.viewport(390, 844);
		await render(LanguageSwitch, { value: 'id', options, onchange: () => {} });
		const pill = page.getByRole('combobox', { name: 'Language' }).element().parentElement!;
		expect(getComputedStyle(pill).height).toBe('44px');
		expect(pill.textContent).toContain('Bahasa Indonesia');
	});
});

describe('GoogleButton', () => {
	it('is the 44 high bordered button with the G, first on the form', async () => {
		await render(GoogleButton, {});
		const button = page.getByRole('button', { name: 'Continue with Google' });
		await expect.element(button).toHaveAttribute('type', 'submit');
		const style = getComputedStyle(button.element());
		expect(style.height).toBe('44px');
		expect(style.borderTopColor).toBe(tokenColor('nm-border-strong'));
		expect(style.backgroundColor).toBe(tokenBackground('nm-surface'));
		expect(page.getByText('last used').elements()).toHaveLength(0);
	});

	it('marks the way the person signed in last time, in mono accent', async () => {
		await render(GoogleButton, { lastUsed: true });
		const tag = page.getByText('last used');
		await expect.element(tag).toBeVisible();
		expect(getComputedStyle(tag.element()).color).toBe(tokenColor('nm-accent'));
		expect(getComputedStyle(tag.element()).fontFamily).toContain('Geist Mono');
	});

	it('shows it is on its way to Google', async () => {
		await render(GoogleButton, { loading: true });
		const button = page.getByRole('button', { name: 'Continue with Google' });
		await expect.element(button).toBeDisabled();
		await expect.element(button).toHaveAttribute('aria-busy', 'true');
	});
});

describe('OrDivider', () => {
	it('reads "or" between two lines, as the mono label in capitals', async () => {
		await render(OrDivider, {});
		const word = page.getByText('or', { exact: true });
		expect(getComputedStyle(word.element()).textTransform).toBe('uppercase');
		expect(getComputedStyle(word.element()).fontFamily).toContain('Geist Mono');
	});
});

describe('AuthField', () => {
	it('labels a 44 high field with its icon, in the 15px body size', async () => {
		await render(AuthField, { label: 'Email', type: 'email', icon: Mail, placeholder: 'you@example.com', name: 'email' });
		const input = page.getByRole('textbox', { name: 'Email' });
		await expect.element(input).toHaveAttribute('placeholder', 'you@example.com');
		await expect.element(input).toHaveAttribute('name', 'email');
		const field = input.element().parentElement!;
		expect(getComputedStyle(field).height).toBe('44px');
		expect(getComputedStyle(field).borderTopColor).toBe(tokenColor('nm-border-strong'));
		expect(field.querySelector('svg')).not.toBeNull();
		expect(getComputedStyle(input.element()).fontSize).toBe('15px');
	});

	it('takes typing with the pen focus: accent edge and the soft glow', async () => {
		await render(AuthField, { label: 'Email' });
		const input = page.getByRole('textbox', { name: 'Email' });
		await userEvent.click(input);
		await userEvent.keyboard('ana@example.com');
		await expect.element(input).toHaveValue('ana@example.com');
		const field = getComputedStyle(input.element().parentElement!);
		expect(field.borderTopColor).toBe(tokenColor('nm-accent'));
		expect(field.boxShadow).toContain('0px 0px 8px');
	});

	it('says what is wrong in place of the helper', async () => {
		await render(AuthField, { label: 'Password', type: 'password', helper: 'At least 8 characters.', error: 'Use at least 8 characters.' });
		const input = page.getByLabelText('Password');
		await expect.element(input).toHaveAttribute('aria-invalid', 'true');
		await expect.element(input).toHaveAccessibleDescription('Use at least 8 characters.');
		expect(getComputedStyle(input.element().parentElement!).borderTopColor).toBe(tokenColor('nm-danger'));
		expect(page.getByText('At least 8 characters.', { exact: true }).elements()).toHaveLength(0);
	});

	it('uses 16px text on a phone so the page does not zoom', async () => {
		await page.viewport(390, 844);
		await render(AuthField, { label: 'Email' });
		expect(getComputedStyle(page.getByRole('textbox', { name: 'Email' }).element()).fontSize).toBe('16px');
	});
});

describe('AuthButton', () => {
	it('is the 44 high accent action across the form', async () => {
		await render(AuthButton, { label: 'Sign in' });
		const button = page.getByRole('button', { name: 'Sign in' });
		await expect.element(button).toHaveAttribute('type', 'submit');
		const style = getComputedStyle(button.element());
		expect(style.height).toBe('44px');
		expect(style.backgroundColor).toBe(tokenBackground('nm-accent'));
		expect(style.color).toBe(tokenColor('nm-accent-ink'));
	});

	it('shows the work while it runs, and takes no second click', async () => {
		const onclick = vi.fn();
		await render(AuthButton, { label: 'Sign in', loading: true, type: 'button', onclick });
		const button = page.getByRole('button', { name: 'Sign in' });
		await expect.element(button).toBeDisabled();
		await expect.element(button).toHaveAttribute('aria-busy', 'true');
	});

	it('can be a quiet bordered link instead', async () => {
		await render(AuthButton, { label: 'I have an account', href: '/sign-in', variant: 'secondary' });
		const link = page.getByRole('link', { name: 'I have an account' });
		await expect.element(link).toHaveAttribute('href', '/sign-in');
		expect(getComputedStyle(link.element()).borderTopColor).toBe(tokenColor('nm-border-strong'));
	});
});

describe('MailAppButton', () => {
	it('opens the person’s webmail in a new tab', async () => {
		await render(MailAppButton, { app: 'gmail' });
		const link = page.getByRole('link', { name: 'Open Gmail' });
		await expect.element(link).toHaveAttribute('href', 'https://mail.google.com/mail/u/0/#inbox');
		await expect.element(link).toHaveAttribute('target', '_blank');
		await expect.element(link).toHaveAttribute('rel', 'noopener noreferrer');
		expect(getComputedStyle(link.element()).height).toBe('40px');
	});

	it('knows Outlook too', async () => {
		await render(MailAppButton, { app: 'outlook' });
		await expect.element(page.getByRole('link', { name: 'Open Outlook' })).toHaveAttribute('href', 'https://outlook.live.com/mail/0/');
	});
});

describe('AuthNote', () => {
	it('is Nomi saying one helpful thing, on the quiet accent tint', async () => {
		await render(AuthNote, { text: 'New to Nomi? After the code you’ll pick a name and meet the crew.' });
		const text = page.getByText('New to Nomi?', { exact: false });
		const note = text.element().parentElement!;
		expect(getComputedStyle(note).backgroundColor).toBe(tokenBackground('nm-accent-bg'));
		expect(note.querySelector('svg')?.getAttribute('width')).toBe('20');
		expect(note.getAttribute('role')).toBeNull();
	});

	it('announces itself when it reports what just happened', async () => {
		await render(AuthNote, { text: 'Password changed.', announce: true });
		await expect.element(page.getByRole('status')).toHaveTextContent('Password changed.');
	});
});

describe('AuthLink', () => {
	it('is a caption link, or a button when it does something here', async () => {
		const onclick = vi.fn();
		await render(AuthLink, { label: 'Forgot password?', href: '/forgot-password' });
		const link = page.getByRole('link', { name: 'Forgot password?' });
		expect(getComputedStyle(link.element()).fontSize).toBe('12px');
		expect(getComputedStyle(link.element()).color).toBe(tokenColor('nm-text-2'));
		document.body.innerHTML = '';
		await render(AuthLink, { label: 'Start over', onclick });
		await page.getByRole('button', { name: 'Start over' }).click();
		expect(onclick).toHaveBeenCalled();
	});

	it('is a 44px target on a phone', async () => {
		await page.viewport(390, 844);
		await render(AuthLink, { label: 'Forgot password?', href: '/forgot-password' });
		expect(box(page.getByRole('link', { name: 'Forgot password?' }).element()).height).toBe(44);
	});
});
