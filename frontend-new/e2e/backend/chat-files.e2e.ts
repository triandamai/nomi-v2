import { expect, test, type Page } from '@playwright/test';
import postgres from 'postgres';
import { api, createAccount, signInAs, type Account } from './support/accounts';
import { DATABASE_URL } from './support/env';

// Files in a chat against the real backend (UX contract audit-chat-files, UXP-001): uploading,
// an upload that fails mid-way (G3), a file the server refuses (G4), a picture the model can't see
// (G6), a sent file opened (G12), and a shared link naming its files privately (G16).

const IPHONE = 'Mozilla/5.0 (iPhone; CPU iPhone OS 18_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Mobile/15E148 Safari/604.1';
const REPLY = 'This is a fake response for local development and testing.';
const MODEL = 'E2E default (fake)';

const PDF = (text: string) =>
	Buffer.from(
		`%PDF-1.4\n1 0 obj<</Type/Catalog/Pages 2 0 R>>endobj 2 0 obj<</Type/Pages/Kids[3 0 R]/Count 1>>endobj 3 0 obj<</Type/Page/Parent 2 0 R/MediaBox[0 0 200 200]/Contents 4 0 R/Resources<</Font<</F1 5 0 R>>>>>>endobj 4 0 obj<</Length 44>>stream\nBT /F1 12 Tf 20 100 Td (${text}) Tj ET\nendstream endobj 5 0 obj<</Type/Font/Subtype/Type1/BaseFont/Helvetica>>endobj\ntrailer<</Root 1 0 R>>\n%%EOF`,
	);
const PNG = Buffer.from('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==', 'base64');
const EXE = Buffer.from('MZ\x90\x00\x03\x00\x00\x00\x04\x00\x00\x00\xff\xff\x00\x00PE\0\0', 'latin1');

const json = async <T>(response: Promise<Response>): Promise<T> => (await response).json() as Promise<T>;

async function newChat(account: Account): Promise<string> {
	return (await json<{ session_id: string }>(api(account, '/api/sessions', { method: 'POST' }))).session_id;
}

async function open(page: Page, path: string) {
	await page.goto(path, { waitUntil: 'networkidle' });
}

const picker = (page: Page) => page.locator('input[type=file]');
const field = (page: Page) => page.getByRole('textbox', { name: 'Message' });

/** Hides every model that can see images for this person, so the chat's only model can't. */
async function hideVisionModels(account: Account) {
	type Options = { groups: { models: { key: string; capabilities?: string[] }[] }[] };
	const options = await json<Options>(api(account, '/api/llm/model-options'));
	const changes = options.groups.flatMap((group) => group.models).filter((model) => model.capabilities?.includes('vision')).map((model) => ({ key: model.key, enabled: false }));
	if (changes.length) expect((await api(account, '/api/llm/model-options', { method: 'PUT', body: JSON.stringify({ changes }) })).ok).toBe(true);
}

test.describe.configure({ mode: 'serial' });

test('a failed upload keeps Send off until it’s retried or removed; the files go with the message and open beside it (G3, G12)', async ({ page, context }) => {
	const account = await createAccount();
	const chat = await newChat(account);
	await signInAs(context, account);
	await open(page, `/chat/${chat}`);

	// The connection drops for one file: it fails, the others carry on.
	let cut = true;
	await page.route('**/attachments', (route) => (cut && route.request().headers()['x-file-name']?.includes('oct') ? route.abort('connectionreset') : route.continue()));
	await picker(page).setInputFiles([
		{ name: 'bca-sep-2026.pdf', mimeType: 'application/pdf', buffer: PDF('September') },
		{ name: 'bca-oct-2026.pdf', mimeType: 'application/pdf', buffer: PDF('October') },
		{ name: 'bca-nov-2026.pdf', mimeType: 'application/pdf', buffer: PDF('November') },
	]);
	const failed = page.getByRole('group', { name: 'bca-oct-2026.pdf' });
	await expect(failed.getByText('Upload failed · connection lost')).toBeVisible({ timeout: 5000 });
	await expect(page.getByRole('group', { name: 'bca-sep-2026.pdf' }).getByText(/^PDF · \d/)).toBeVisible();
	await expect(page.getByRole('group', { name: 'bca-nov-2026.pdf' }).getByText(/^PDF · \d/)).toBeVisible();

	// Send waits, says why, and Enter doesn't send.
	const sendButton = page.getByRole('button', { name: 'Send', exact: true });
	await expect(page.getByText('1 file didn’t upload. Retry or remove it to send.')).toBeVisible();
	await expect(sendButton).toBeDisabled();
	await expect(sendButton).toHaveAttribute('aria-disabled', 'true');
	await field(page).fill('What did I spend on rent and food?');
	await field(page).press('Enter');
	await expect(field(page)).toHaveValue('What did I spend on rent and food?');

	// Retry starts the upload again.
	cut = false;
	await failed.getByRole('button', { name: 'Retry bca-oct-2026.pdf' }).click();
	await expect(failed.getByText(/^PDF · \d/)).toBeVisible();
	await expect(sendButton).toBeEnabled();

	// Removing a file that didn't upload says what's left; Send is free.
	cut = true;
	await picker(page).setInputFiles([{ name: 'bca-oct-copy.pdf', mimeType: 'application/pdf', buffer: PDF('Copy') }]);
	await expect(page.getByRole('group', { name: 'bca-oct-copy.pdf' }).getByText('Upload failed · connection lost')).toBeVisible();
	await page.getByRole('button', { name: 'Remove bca-oct-copy.pdf' }).click();
	await expect(page.locator('[data-composer]').getByText('bca-oct-copy.pdf removed · 3 files ready')).toBeVisible();
	await expect(sendButton).toBeEnabled();

	await sendButton.click();
	const files = page.getByRole('list', { name: 'Files sent with this message' });
	await expect(files.getByRole('button', { name: 'Open bca-sep-2026.pdf' })).toBeVisible();
	await expect(page.getByRole('main').getByText(REPLY)).toBeVisible();
	type Page_ = { messages: { sender: string; content: string }[] };
	const sent = (await json<Page_>(api(account, `/api/sessions/${chat}/messages`))).messages.find((message) => message.sender === 'user')!;
	expect(sent.content.match(/<attachment /g)).toHaveLength(3);
	expect(sent.content).not.toContain('bca-oct-copy.pdf');

	// A sent PDF opens beside the chat at page 1; Escape closes it and gives focus back.
	const opener = files.getByRole('button', { name: 'Open bca-sep-2026.pdf' });
	await opener.click();
	const viewer = page.getByRole('dialog', { name: 'bca-sep-2026.pdf' });
	await expect(viewer).toBeVisible();
	await expect(viewer.getByText(/^PDF · .* · your attachment$/)).toBeVisible();
	await expect(viewer.getByRole('button', { name: 'Close' })).toBeFocused();
	await expect(viewer.getByRole('link', { name: 'Download bca-sep-2026.pdf' })).toHaveAttribute('href', /\/attachments\/[0-9a-f-]{36}\/content\?download$/);
	await page.keyboard.press('Escape');
	await expect(viewer).toBeHidden();
	await expect(opener).toBeFocused();
});

test('a file the server won’t take is refused at its chip, and Backspace removes it (G4)', async ({ page, context }) => {
	const account = await createAccount();
	const chat = await newChat(account);
	await signInAs(context, account);
	await open(page, `/chat/${chat}`);

	await picker(page).setInputFiles([{ name: 'kasir-setup.exe', mimeType: 'application/octet-stream', buffer: EXE }]);
	const refused = page.getByRole('group', { name: 'kasir-setup.exe' });
	await expect(refused.getByText('.exe files can’t be read')).toBeVisible();
	await expect(page.getByText('Remove kasir-setup.exe to send.')).toBeVisible();
	await expect(refused.getByRole('button', { name: /^Retry/ })).toHaveCount(0);
	await refused.focus();
	await page.keyboard.press('Backspace');
	await expect(refused).toHaveCount(0);
	await expect(field(page)).toBeFocused();
	await expect(page.locator('[data-composer]').getByText('kasir-setup.exe removed')).toBeVisible();
});

test('a picture the model can’t see: the warning, no model to switch to, the draft kept through Settings, then text only (G6)', async ({ page, context }) => {
	const account = await createAccount();
	await hideVisionModels(account);
	const chat = await newChat(account);
	await signInAs(context, account);
	await open(page, `/chat/${chat}`);

	await field(page).fill('What’s the total on this receipt?');
	await picker(page).setInputFiles([{ name: 'struk-belanja-nov.png', mimeType: 'image/png', buffer: PNG }]);
	const chip = page.getByRole('group', { name: 'struk-belanja-nov.png' });
	await expect(chip.getByText(`${MODEL} can’t see images`)).toBeVisible();
	await expect(page.getByText('Switch to a model that can see images, or send your message without the photo.')).toBeVisible();
	await expect(page.getByRole('button', { name: 'Switch model' })).toBeFocused();
	await expect(page.getByRole('button', { name: 'Send', exact: true })).toBeDisabled();

	await page.getByRole('button', { name: 'Switch model' }).click();
	const menu = page.getByRole('menu', { name: 'No model here can see images' });
	await expect(menu.getByText(`${MODEL} hidden · can’t see images`)).toBeVisible();
	await expect(menu.getByRole('menuitem', { name: 'Add one in Models & providers' })).toBeFocused();
	await page.keyboard.press('Escape');
	await expect(menu).toBeHidden();
	await expect(page.getByRole('button', { name: 'Switch model' })).toBeFocused();

	// To Settings and back: the draft and the picture are still there.
	await page.getByRole('button', { name: 'Switch model' }).click();
	await menu.getByRole('menuitem', { name: 'Add one in Models & providers' }).click();
	await expect(page).toHaveURL(/\/settings\/models$/);
	await page.goBack({ waitUntil: 'networkidle' });
	await expect(field(page)).toHaveValue('What’s the total on this receipt?');
	await expect(page.getByRole('group', { name: 'struk-belanja-nov.png' }).getByText(`${MODEL} can’t see images`)).toBeVisible();

	// Send text only: the message goes without the picture, and says so.
	await page.getByRole('button', { name: 'Send text only' }).click();
	await expect(page.getByText(`Sent without struk-belanja-nov.png · ${MODEL} can’t see images`)).toBeVisible();
	await expect(page.getByRole('main').getByText(REPLY)).toBeVisible();
	type Page_ = { messages: { sender: string; content: string }[] };
	const sent = (await json<Page_>(api(account, `/api/sessions/${chat}/messages`))).messages.find((message) => message.sender === 'user')!;
	expect(sent.content).toBe('What’s the total on this receipt?');
});

test('Switch model lists only the models that can see, counts the rest, and picking one clears the warning (G6)', async ({ page, context }) => {
	const account = await createAccount();
	const label = `E2E vision ${Math.random().toString(36).slice(2, 8)}`;
	const sql = postgres(DATABASE_URL, { max: 1, onnotice: () => {} });
	const [made] = await sql`
		INSERT INTO admin_llm_models (label, provider, model_id, api_key_encrypted, is_default, media_inputs, updated_by)
		SELECT ${label}, 'fake', 'fake', ''::bytea, false, ARRAY['image'], user_id FROM web_credentials WHERE email = ${account.email}
		RETURNING id`;
	try {
		const chat = await newChat(account);
		await signInAs(context, account);
		await open(page, `/chat/${chat}`);
		await picker(page).setInputFiles([{ name: 'struk.png', mimeType: 'image/png', buffer: PNG }]);
		await expect(page.getByRole('group', { name: 'struk.png' }).getByText(`${MODEL} can’t see images`)).toBeVisible();
		await page.getByRole('button', { name: 'Switch model' }).click();
		const menu = page.getByRole('menu', { name: 'Switch model' });
		await expect(menu.getByRole('menuitem', { name: new RegExp(label) })).toBeVisible();
		await expect(menu.getByRole('menuitem', { name: new RegExp(MODEL.replace(/[()]/g, '\\$&')) })).toHaveCount(0);
		await expect(menu.getByText(`${MODEL} hidden · can’t see images`)).toBeVisible();
		await menu.getByRole('menuitem', { name: new RegExp(label) }).click();
		await expect(page.getByText('Switch to a model that can see images', { exact: false })).toHaveCount(0);
		await expect(page.getByRole('button', { name: 'Send', exact: true })).toBeEnabled();

		// Sent, the photo opens large (G12 #28): zoom, Download, and Escape closes it.
		await page.getByRole('button', { name: 'Send', exact: true }).click();
		const photo = page.getByRole('button', { name: 'Open struk.png' });
		await photo.click();
		const lightbox = page.getByRole('dialog', { name: 'struk.png' });
		await expect(lightbox.getByText('1 of 1 photo in this message')).toBeVisible();
		await expect(lightbox.getByText('esc closes')).toBeVisible();
		await lightbox.getByRole('button', { name: 'Zoom in' }).click();
		await expect(lightbox.getByText('125%')).toBeVisible();
		await page.keyboard.press('Escape');
		await expect(lightbox).toBeHidden();
		await expect(photo).toBeFocused();
	} finally {
		await deleteModel(made.id as string);
		await sql.end();
	}
});

async function deleteModel(id: string) {
	const sql = postgres(DATABASE_URL, { max: 1, onnotice: () => {} });
	try {
		await sql`DELETE FROM chat_model_choices WHERE model_key = ${`admin:${id}`}`.catch(() => undefined);
		await sql`DELETE FROM admin_llm_models WHERE id = ${id}`;
	} finally {
		await sql.end();
	}
}

test('a shared link names the files a message went with, locked and private, and never opens one (G16)', async ({ page, context, browser }) => {
	const account = await createAccount();
	const chat = await newChat(account);
	await signInAs(context, account);
	await open(page, `/chat/${chat}`);
	await picker(page).setInputFiles([{ name: 'bca-sep-2026.pdf', mimeType: 'application/pdf', buffer: PDF('September') }]);
	await expect(page.getByRole('group', { name: 'bca-sep-2026.pdf' }).getByText(/^PDF · \d/)).toBeVisible();
	await field(page).fill('What did I spend in September?');
	await page.getByRole('button', { name: 'Send', exact: true }).click();
	await expect(page.getByRole('main').getByText(REPLY)).toBeVisible();

	const share = await json<{ token: string }>(api(account, `/api/sessions/${chat}/shares`, { method: 'POST', body: JSON.stringify({ scope: 'chat' }) }));
	const visitor = await browser.newContext();
	const viewer = await visitor.newPage();
	await viewer.goto(`/shared/${share.token}`);
	await expect(viewer.getByText('bca-sep-2026.pdf')).toBeVisible();
	await expect(viewer.getByText('PDF · private')).toBeVisible();
	await expect(viewer.getByText(/files private/)).toBeVisible();
	await expect(viewer.getByText(/kept the files in this reply private\. You see their names only\./)).toBeVisible();
	await expect(viewer.getByRole('button', { name: /bca-sep-2026/ })).toHaveCount(0);
	expect(await viewer.locator('a[href*="/attachments/"]').count()).toBe(0);
	expect(await viewer.content()).not.toMatch(/\/attachments\/[0-9a-f-]{36}/);
	await visitor.close();
});

test.describe('on a phone', () => {
	test.use({ viewport: { width: 390, height: 844 }, userAgent: IPHONE, hasTouch: true });

	test('a failed file opens its sheet: Retry starts the upload again, Remove keeps it on the phone (G3)', async ({ page, context }) => {
		const account = await createAccount();
		const chat = await newChat(account);
		await signInAs(context, account);
		await open(page, `/chat/${chat}`);

		let cut = true;
		await page.route('**/attachments', (route) => (cut ? route.abort('connectionreset') : route.continue()));
		await picker(page).setInputFiles([{ name: 'bca-oct-2026.pdf', mimeType: 'application/pdf', buffer: PDF('October') }]);
		await expect(page.getByText('Upload failed · connection lost')).toBeVisible();
		await expect(page.getByText('1 file didn’t upload. Retry or remove it to send.')).toBeVisible();
		await page.getByRole('button', { name: 'Retry or remove bca-oct-2026.pdf' }).click();
		const sheet = page.getByRole('dialog', { name: 'bca-oct-2026.pdf' });
		await expect(sheet.getByRole('heading', { name: 'bca-oct-2026.pdf' })).toBeVisible();
		await expect(sheet.getByText('starts the upload again')).toBeVisible();
		await expect(sheet.getByText('stays on your phone')).toBeVisible();
		cut = false;
		await sheet.getByRole('button', { name: /^Retry/ }).click();
		await expect(sheet).toBeHidden();
		await expect(page.getByRole('group', { name: 'bca-oct-2026.pdf' }).getByText(/^PDF · \d/)).toBeVisible();
	});

	test('a picture the model can’t see: the pills, and the sheet focuses Add one (G6, M13 · M15)', async ({ page, context }) => {
		const account = await createAccount();
		await hideVisionModels(account);
		const chat = await newChat(account);
		await signInAs(context, account);
		await open(page, `/chat/${chat}`);
		await picker(page).setInputFiles([{ name: 'struk.png', mimeType: 'image/png', buffer: PNG }]);
		await expect(page.getByText('Switch to a model that can see images, or send without the photo.')).toBeVisible();
		const pill = page.getByRole('button', { name: 'Switch model' });
		expect((await pill.boundingBox())?.height).toBe(32);
		expect((await page.getByRole('group', { name: 'struk.png' }).locator('> span').boundingBox())?.height).toBe(43);
		await pill.click();
		const sheet = page.getByRole('dialog', { name: 'No model here can see images' });
		await expect(sheet.getByRole('link', { name: 'Add one in Models & providers' })).toBeFocused();
	});

	test('a sent PDF opens full screen (G12)', async ({ page, context }) => {
		const account = await createAccount();
		const chat = await newChat(account);
		await signInAs(context, account);
		await open(page, `/chat/${chat}`);
		await picker(page).setInputFiles([{ name: 'bca-sep-2026.pdf', mimeType: 'application/pdf', buffer: PDF('September') }]);
		await expect(page.getByRole('group', { name: 'bca-sep-2026.pdf' }).getByText(/^PDF · \d/)).toBeVisible();
		await page.getByRole('button', { name: 'Send', exact: true }).click();
		await page.getByRole('button', { name: 'Open bca-sep-2026.pdf' }).click();
		const viewer = page.getByRole('dialog', { name: 'bca-sep-2026.pdf' });
		await expect(viewer.getByText(/· yours$/)).toBeVisible();
		expect((await viewer.boundingBox())?.width).toBe(390);
		await viewer.getByRole('button', { name: 'Close' }).click();
		await expect(viewer).toBeHidden();
	});
});
