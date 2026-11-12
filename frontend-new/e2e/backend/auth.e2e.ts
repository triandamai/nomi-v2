import { expect, test, type BrowserContext, type Page } from '@playwright/test';
import { api, createAccount, newEmail, signInAs, type Account } from './support/accounts';
import { BACKEND_URL } from './support/env';
import { codeFor, linkFor } from './support/mailpit';

// Sign in and create an account against the real backend: codes and links come from Mailpit, as
// the person would read them. Cases the backend can't be made to reach here (a code expiring after
// 10 minutes, resend running out of codes, Google's own callback) are covered by the pages' unit
// tests with the backend's answers.

/** A code that isn't `code`. */
const wrong = (code: string) => String((Number(code) + 1) % 1_000_000).padStart(6, '0');

/** Whether the browser is signed in on the backend: its cookie works there. */
async function signedInAs(context: BrowserContext): Promise<string | null> {
	const token = (await context.cookies()).find((cookie) => cookie.name === 'access_token')?.value;
	if (!token) return null;
	const response = await fetch(`${BACKEND_URL}/api/whoami`, { headers: { authorization: `Bearer ${token}` } });
	return response.ok ? ((await response.json()) as { sub: string }).sub : null;
}

async function signInWithPassword(page: Page, account: Account, path = '/sign-in') {
	await page.goto(path);
	await page.getByLabel('Email').fill(account.email);
	await page.getByLabel('Password').fill(account.password);
	await page.getByRole('button', { name: 'Sign in' }).click();
	await expect(page).toHaveURL(/\/verify$/);
}

/** Types the newest code of `subject` sent to `email` since `after`. */
async function typeCode(page: Page, email: string, subject: RegExp, after: number) {
	const code = await codeFor(email, subject, { after: after - 1000 });
	await page.getByRole('textbox', { name: 'Digit 1 of 6' }).click();
	await page.keyboard.type(code);
	return code;
}

test.describe('create an account', () => {
	test('register, the emailed code, a name, and in', async ({ page, context }) => {
		const email = newEmail('register');
		await page.goto('/register');
		await expect(page.getByRole('button', { name: 'Continue with Google' })).toHaveCount(0);
		await page.getByLabel('Email').fill(email);
		await page.getByLabel('Password').fill('correct-horse-9');
		await page.getByLabel('Workspace name').fill('Family budget');
		const started = Date.now();
		await page.getByRole('button', { name: 'Create account' }).click();

		await expect(page).toHaveURL(/\/verify$/);
		await expect(page.getByRole('heading', { name: 'Check your email' })).toBeVisible();
		await expect(page.getByText(/We sent a 6-digit code to r•+@nomi\.test\. It works for 10 minutes\./)).toBeVisible();
		await expect(page.getByText('New to Nomi? After the code you’ll pick a name and meet the crew.', { exact: false })).toBeVisible();
		await expect(page.getByRole('timer')).toHaveText(/resend in 0:[0-5]\d/);
		await typeCode(page, email, /confirm/i, started);

		await expect(page).toHaveURL(/\/first-run$/);
		const name = page.getByRole('textbox', { name: 'Your name' });
		await expect(name).toHaveValue(email.split('@')[0]);
		await name.fill('Ana Wijaya');
		await page.getByRole('button', { name: 'Continue' }).click();
		await expect(page).toHaveURL(/\/$/);

		expect(await signedInAs(context)).toMatch(/^[0-9a-f-]{36}$/);
		const token = (await context.cookies()).find((cookie) => cookie.name === 'access_token')!.value;
		const profile = (await (await fetch(`${BACKEND_URL}/api/profile`, { headers: { authorization: `Bearer ${token}` } })).json()) as { display_name: string };
		expect(profile.display_name).toBe('Ana Wijaya');
	});

	test('an address that already has an account is turned away on its field', async ({ page }) => {
		const account = await createAccount();
		await page.goto('/register');
		await page.getByLabel('Email').fill(account.email);
		await page.getByLabel('Password').fill('another-horse-9');
		await page.getByLabel('Workspace name').fill('Second');
		await page.getByRole('button', { name: 'Create account' }).click();
		await expect(page.getByText('There’s already an account with this email. Sign in instead.')).toBeVisible();
		await expect(page.getByLabel('Email')).toHaveAttribute('aria-invalid', 'true');
		await expect(page).toHaveURL(/\/register$/);
	});

	test('a short password and a missing workspace are caught before anything is sent', async ({ page }) => {
		await page.goto('/register');
		await page.getByLabel('Email').fill(newEmail('short'));
		await page.getByLabel('Password').fill('short');
		await page.getByRole('button', { name: 'Create account' }).click();
		await expect(page.getByText('Use at least 8 characters.')).toBeVisible();
		await expect(page.getByText('Name your workspace.')).toBeVisible();
	});
});

test.describe('sign in', () => {
	test('the password, then the code, and on to where the person was going', async ({ page, context }) => {
		const account = await createAccount();
		const started = Date.now();
		await signInWithPassword(page, account, '/sign-in?redirect_to=%2Ffirst-run');
		await typeCode(page, account.email, /sign-in code/i, started);
		await expect(page).toHaveURL(/\/first-run$/);
		await expect(page.getByRole('heading', { name: 'What should we call you?' })).toBeVisible();
		expect(await signedInAs(context)).not.toBeNull();
	});

	test('an empty or mistyped email, and an empty password, are said on their fields', async ({ page }) => {
		await page.goto('/sign-in');
		await page.getByRole('button', { name: 'Sign in' }).click();
		await expect(page.getByText('Enter your email address.')).toBeVisible();
		await expect(page.getByText('Enter your password.')).toBeVisible();
		await page.getByLabel('Email').fill('ana');
		await page.getByRole('button', { name: 'Sign in' }).click();
		await expect(page.getByText('That doesn’t look like an email address.')).toBeVisible();
	});

	test('a wrong password says so on the password, keeping the email', async ({ page }) => {
		const account = await createAccount();
		await page.goto('/sign-in');
		await page.getByLabel('Email').fill(account.email);
		await page.getByLabel('Password').fill('not-the-password');
		await page.getByRole('button', { name: 'Sign in' }).click();
		await expect(page.getByText('That email and password don’t match. Check both, or reset your password.')).toBeVisible();
		await expect(page.getByLabel('Email')).toHaveValue(account.email);
		await expect(page).toHaveURL(/\/sign-in$/);
	});

	test('a wrong code says how many tries are left; five lock it, and a reload says it has ended', async ({ page }) => {
		const account = await createAccount();
		const started = Date.now();
		await signInWithPassword(page, account);
		const code = await codeFor(account.email, /sign-in code/i, { after: started - 1000 });
		const first = page.getByRole('textbox', { name: 'Digit 1 of 6' });

		for (const left of [4, 3, 2]) {
			await first.click();
			await page.keyboard.type(wrong(code));
			await expect(page.getByRole('alert')).toHaveText(`That code isn’t right. ${left} tries left.`);
			await expect(first).toHaveValue('');
		}
		await first.click();
		await page.keyboard.type(wrong(code));
		await expect(page.getByRole('alert')).toHaveText('That code isn’t right. 1 try left.');
		await first.click();
		await page.keyboard.type(wrong(code));
		await expect(page.getByText('Too many wrong codes, so this one is locked. Start over to get a new one.')).toBeVisible();
		await expect(page.getByRole('textbox', { name: 'Digit 1 of 6' })).toHaveCount(0);

		await page.reload();
		await expect(page.getByText('This code has expired. Start over to get a new one.')).toBeVisible();
		await page.getByRole('button', { name: 'Start over' }).click();
		await expect(page).toHaveURL(/\/sign-in$/);
	});

	test('asking for a new code too soon shows the wait the backend gives', async ({ page }) => {
		const account = await createAccount();
		await page.clock.install();
		await signInWithPassword(page, account);
		await expect(page.getByRole('timer')).toHaveText(/resend in 0:[0-5]\d/);
		// The page's clock runs ahead; the backend's doesn't, so it says to wait.
		await page.clock.fastForward('01:05');
		await page.getByRole('button', { name: 'send a new code' }).click();
		await expect(page.getByText('Too soon for a new code. Try again when the timer ends.')).toBeVisible();
		await expect(page.getByRole('timer')).toHaveText(/resend in 0:[0-5]\d/);
	});

	test('too many codes in an hour is said plainly', async ({ page }) => {
		const account = await createAccount();
		// Registering sent one; nine more sign-ins make the ten an account may have in an hour.
		for (let index = 0; index < 9; index++) {
			const response = await fetch(`${BACKEND_URL}/api/auth/login`, {
				method: 'POST',
				headers: { 'content-type': 'application/json' },
				body: JSON.stringify({ email: account.email, password: account.password }),
			});
			expect(response.status).toBe(200);
		}
		await page.goto('/sign-in');
		await page.getByLabel('Email').fill(account.email);
		await page.getByLabel('Password').fill(account.password);
		await page.getByRole('button', { name: 'Sign in' }).click();
		await expect(page.getByRole('alert')).toContainText('We’ve sent this account too many codes in the last hour.');
		await expect(page.getByRole('alert')).toContainText('429 · too_many_codes');
	});

	test('change goes back to the form', async ({ page }) => {
		const account = await createAccount();
		await signInWithPassword(page, account, '/sign-in?redirect_to=%2Ffirst-run');
		await page.getByRole('button', { name: /Change the email/ }).click();
		await expect(page).toHaveURL(/\/sign-in\?redirect_to=%2Ffirst-run$/);
	});

	test('Google is left out when the server has no Google sign-in', async ({ page }) => {
		for (const path of ['/sign-in', '/register']) {
			await page.goto(path);
			await expect(page.getByRole('button', { name: 'Continue with Google' })).toHaveCount(0);
			await expect(page.getByText('or', { exact: true })).toHaveCount(0);
		}
	});

	test('the language switch shows the screens in Indonesian', async ({ page }) => {
		await page.goto('/sign-in');
		await page.getByRole('combobox', { name: 'Language' }).selectOption('id');
		await expect(page.getByRole('heading', { name: 'Masuk ke Nomi' })).toBeVisible();
		await expect(page.locator('html')).toHaveAttribute('lang', 'id');
	});
});

test.describe('forgot password', () => {
	test('the emailed link, a new password, and signing in with it', async ({ page, context }) => {
		const account = await createAccount();
		await page.goto('/sign-in');
		await page.getByRole('link', { name: 'Forgot password?' }).click();
		// The sign-in page has an Email field too: wait for the new page before typing.
		await expect(page.getByRole('heading', { name: 'Reset your password' })).toBeVisible();
		await page.getByLabel('Email').fill(account.email);
		await page.getByRole('button', { name: 'Send reset link' }).click();
		await expect(page.getByText(`If there’s an account for ${account.email}, a link to choose a new password is on its way.`, { exact: false })).toBeVisible();

		// From the path on: the backend's APP_URL may name another port.
		await page.goto(await linkFor(account.email, '/reset-password?token='));
		await expect(page.getByRole('heading', { name: 'Choose a new password' })).toBeVisible();
		await page.getByLabel('New password', { exact: true }).fill('brand-new-horse');
		await page.getByLabel('Confirm new password').fill('brand-new-hors');
		await page.getByRole('button', { name: 'Save new password' }).click();
		await expect(page.getByText('The two passwords don’t match.')).toBeVisible();
		// Until the page has refreshed after that answer, the button stays busy.
		await expect(page.getByRole('button', { name: 'Save new password' })).toBeEnabled();
		await page.getByLabel('New password', { exact: true }).fill('short');
		await page.getByLabel('Confirm new password').fill('short');
		await page.getByRole('button', { name: 'Save new password' }).click();
		await expect(page.getByText('Use at least 8 characters.')).toBeVisible();
		await expect(page.getByRole('button', { name: 'Save new password' })).toBeEnabled();
		await page.getByLabel('New password', { exact: true }).fill('brand-new-horse');
		await page.getByLabel('Confirm new password').fill('brand-new-horse');
		await page.getByRole('button', { name: 'Save new password' }).click();

		await expect(page).toHaveURL(/\/sign-in\?reset=1$/);
		await expect(page.getByRole('status')).toHaveText('Password changed, and every device was signed out. Sign in with your new password.');
		const started = Date.now();
		await signInWithPassword(page, { ...account, password: 'brand-new-horse' });
		await typeCode(page, account.email, /sign-in code/i, started);
		await expect(page).toHaveURL(/\/$/);
		expect(await signedInAs(context)).not.toBeNull();
	});

	test('a link that is unknown or already used says it has expired', async ({ page }) => {
		await page.goto('/reset-password?token=not-a-real-token');
		await expect(page.getByRole('heading', { name: 'This link has expired' })).toBeVisible();
		await page.getByRole('link', { name: 'Send a new link' }).click();
		await expect(page).toHaveURL(/\/forgot-password$/);
	});
});

test.describe('invites', () => {
	async function inviteFrom(owner: Account, email: string): Promise<string> {
		const orgs = (await (await api(owner, '/api/orgs')).json()) as { id: string }[] | { orgs: { id: string }[] };
		const [org] = Array.isArray(orgs) ? orgs : orgs.orgs;
		const response = await api(owner, `/api/orgs/${org.id}/invites`, { method: 'POST', body: JSON.stringify({ email }) });
		expect(response.status).toBe(201);
		return ((await response.json()) as { code: string }).code;
	}

	test('someone new opens the invite signed out, signs up from the email, and is in the workspace', async ({ page, context }) => {
		const owner = await createAccount({ workspace: 'Warung Bu Sari' });
		const email = newEmail('invitee');
		const code = await inviteFrom(owner, email);

		await page.goto(`/invite/${code}`);
		await expect(page.getByRole('heading', { name: 'You’re invited to Nomi' })).toBeVisible();
		await expect(page.getByRole('link', { name: 'Create an account' })).toHaveAttribute('href', `/register?redirect_to=%2Finvite%2F${code}`);

		// The invite email links to sign-up with the address filled in.
		await page.goto(await linkFor(email, '/register?redirect_to='));
		await expect(page.getByLabel('Email')).toHaveValue(email);
		await expect(page.getByLabel('Invite code')).toHaveValue(code);
		await expect(page.getByLabel('Workspace name')).toHaveCount(0);
		await page.getByLabel('Password').fill('invited-horse-9');
		const started = Date.now();
		await page.getByRole('button', { name: 'Create account' }).click();
		await typeCode(page, email, /confirm/i, started);
		await expect(page).toHaveURL(new RegExp(`/first-run\\?redirect_to=%2Finvite%2F${code}$`));
		await page.getByRole('button', { name: 'Continue' }).click();

		await expect(page).toHaveURL(new RegExp(`/invite/${code}$`));
		await expect(page.getByRole('heading', { name: 'You’re already in Warung Bu Sari' })).toBeVisible();
		expect(await signedInAs(context)).not.toBeNull();
	});

	test('someone with an account signs in from the invite and joins', async ({ page, context }) => {
		const owner = await createAccount({ workspace: 'Kopi Kenangan Kita' });
		const guest = await createAccount();
		const code = await inviteFrom(owner, guest.email);

		await page.goto(`/invite/${code}`);
		await page.getByRole('link', { name: 'I have an account' }).click();
		const started = Date.now();
		await page.getByLabel('Email').fill(guest.email);
		await page.getByLabel('Password').fill(guest.password);
		await page.getByRole('button', { name: 'Sign in' }).click();
		await typeCode(page, guest.email, /sign-in code/i, started);

		await expect(page).toHaveURL(new RegExp(`/invite/${code}$`));
		await expect(page.getByRole('heading', { name: 'Join Kopi Kenangan Kita' })).toBeVisible();
		await expect(page.getByText(/invited you to Kopi Kenangan Kita as member\./)).toBeVisible();
		await page.getByRole('button', { name: 'Join Kopi Kenangan Kita' }).click();
		await expect(page).toHaveURL(/\/$/);
		expect(await signedInAs(context)).not.toBeNull();

		await page.goto(`/invite/${code}`);
		await expect(page.getByRole('heading', { name: 'You’re already in Kopi Kenangan Kita' })).toBeVisible();
	});

	test('an invite that does not exist says so', async ({ page, context }) => {
		await signInAs(context, await createAccount(), test.info().project.use.baseURL);
		await page.goto('/invite/NoSuchInviteCode123');
		await expect(page.getByRole('heading', { name: 'We can’t find this invite' })).toBeVisible();
	});
});
