import { expect, test, type Page } from '@playwright/test';
import { api, createAccount, newEmail, signInAs, type Account } from './support/accounts';

// Settings › Workspace against the real backend (App / Settings / Workspace wZm9B): an owner
// invites (a bad address turned away first), takes an invite back, changes a member's role and
// removes them, and can't leave as the only owner; an invited member joins, then leaves.

type Orgs = { orgs: { id: string; name: string }[] };
type Detail = { members: { user_id: string; name: string; role: string }[]; invites: { code: string; email: string }[] };
const json = async <T>(response: Promise<Response>) => (await response).json() as Promise<T>;
const workspaceOf = async (account: Account) => (await json<Orgs>(api(account, '/api/orgs'))).orgs[0];

async function openWorkspace(page: Page, id: string) {
	await page.goto(`/settings/workspaces/${id}`, { waitUntil: 'networkidle' });
}

test('an owner invites, revokes, changes a role and removes someone', async ({ page, context }) => {
	const owner = await createAccount();
	const siti = await createAccount();
	const kopi = await workspaceOf(owner);
	await signInAs(context, owner);
	await page.goto('/settings/workspaces', { waitUntil: 'networkidle' });
	await page.getByRole('article', { name: kopi.name }).getByRole('link', { name: 'Settings' }).click();
	await expect(page).toHaveURL(new RegExp(`/settings/workspaces/${kopi.id}$`));
	await expect(page.getByRole('heading', { level: 1, name: kopi.name })).toBeVisible();
	await expect(page.getByText('workspace · you’re Owner')).toBeVisible();
	await expect(page.getByRole('navigation', { name: 'Settings' }).getByRole('link', { name: 'Workspaces' })).toHaveAttribute('aria-current', 'page');
	await expect(page.getByRole('button', { name: 'Send invites' })).toBeDisabled();

	// A bad address is turned away before anything is sent.
	const field = page.getByRole('textbox', { name: 'Email addresses' });
	await field.fill('nope');
	await page.getByRole('button', { name: 'Send invite' }).click();
	await expect(page.getByText('Check these addresses: nope')).toBeVisible();

	// Two addresses, as admins.
	const stranger = newEmail('stranger');
	await page.getByRole('button', { name: 'Remove nope' }).click();
	await field.fill(`${siti.email}, ${stranger}`);
	await page.getByText('Admin', { exact: true }).click();
	await expect(page.getByRole('radio', { name: /^Admin/ })).toBeChecked();
	await page.getByRole('button', { name: 'Send 2 invites' }).click();
	await expect(page.getByText('2 invites sent')).toBeVisible();
	await expect(page.getByText(stranger)).toBeVisible();
	await expect(page.getByText('pending · Admin')).toHaveCount(2);

	// The stranger's invite taken back.
	await page.getByRole('button', { name: `Revoke the invite to ${stranger}` }).click();
	await expect(page.getByText(`Invite to ${stranger} revoked`)).toBeVisible();
	await expect(page.getByText(stranger)).toHaveCount(0);

	// Siti joins; the owner makes her a member, then removes her.
	const code = (await json<Detail>(api(owner, `/api/orgs/${kopi.id}`))).invites.find((one) => one.email === siti.email)!.code;
	expect((await api(siti, `/api/invites/${code}/accept`, { method: 'POST' })).ok).toBe(true);
	await openWorkspace(page, kopi.id);
	const sitiName = (await json<Detail>(api(owner, `/api/orgs/${kopi.id}`))).members.find((one) => one.role === 'admin')!.name;
	await page.getByRole('combobox', { name: `${sitiName}’s role` }).selectOption('member');
	await expect(page.getByText(`${sitiName} is now Member`)).toBeVisible();
	await expect.poll(async () => (await json<Detail>(api(owner, `/api/orgs/${kopi.id}`))).members.find((one) => one.name === sitiName)?.role).toBe('member');
	await page.getByRole('button', { name: `Remove ${sitiName}` }).click();
	await page.getByRole('dialog', { name: `Remove ${sitiName}?` }).getByRole('button', { name: 'Remove' }).click();
	await expect(page.getByText(`${sitiName} removed`)).toBeVisible();
	await expect.poll(async () => (await json<Detail>(api(owner, `/api/orgs/${kopi.id}`))).members.length).toBe(1);

	// The only owner can't leave.
	await expect(page.getByText('You’re its only owner: make someone else an owner first.')).toBeVisible();
	await expect(page.getByRole('button', { name: 'Leave' })).toBeDisabled();
});

test('an invited member joins, sees no owner tools, and leaves', async ({ page, context }) => {
	const owner = await createAccount();
	const budi = await createAccount();
	const kopi = await workspaceOf(owner);
	const invited = await json<{ code: string }>(api(owner, `/api/orgs/${kopi.id}/invites`, { method: 'POST', body: JSON.stringify({ email: budi.email, role: 'member' }) }));
	expect((await api(budi, `/api/invites/${invited.code}/accept`, { method: 'POST' })).ok).toBe(true);

	await signInAs(context, budi);
	await openWorkspace(page, kopi.id);
	await expect(page.getByText('workspace · you’re Member')).toBeVisible();
	await expect(page.getByRole('heading', { name: 'Invite people' })).toHaveCount(0);
	await expect(page.getByRole('button', { name: 'Rename' })).toHaveCount(0);
	await expect(page.getByText('Only an owner can make owners or hand the workspace over.')).toBeVisible();

	await page.getByRole('button', { name: 'Leave' }).click();
	await page.getByRole('dialog', { name: `Leave ${kopi.name}` }).getByRole('button', { name: 'Leave' }).click();
	await expect(page).toHaveURL(/\/settings\/workspaces$/);
	await expect(page.getByText(`You left ${kopi.name}`)).toBeVisible();
	// Every e2e workspace is named alike: what's left is Budi's own.
	await expect(page.getByRole('banner').getByText('1 workspace')).toBeVisible();
	expect((await json<Orgs>(api(budi, '/api/orgs'))).orgs.map((org) => org.id)).not.toContain(kopi.id);

	// Someone outside it finds nothing there.
	await openWorkspace(page, kopi.id);
	await expect(page.getByText('This workspace isn’t there, or you’re not in it.')).toBeVisible();
	await page.getByRole('link', { name: 'Workspaces' }).last().click();
	await expect(page).toHaveURL(/\/settings\/workspaces$/);
});
