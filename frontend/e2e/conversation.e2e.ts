import { expect, test } from '@playwright/test';

// These tests exercise the live WebSocket-relayed reply path end to end. They require
// `cargo run --bin worker` (backend/src/bin/worker.rs) running alongside the backend server
// that playwright.config.ts's webServer entry starts, plus EMQX (backend/docker-compose.yml).
// The worker is not auto-started by Playwright — see
// docs/superpowers/plans/2026-08-17-frontend-realtime-rendering-implementation.md (File
// Structure section) for why. Start it manually before running this file.

async function registerAndStartChat(page: import('@playwright/test').Page): Promise<string> {
	const email = `test-${Date.now()}-${Math.random().toString(36).slice(2)}@example.com`;

	await page.goto('/register');
	await page.getByLabel('Email').fill(email);
	await page.getByLabel('Password').fill('correct horse battery staple');
	await page.getByLabel('Organization name').fill('Acme');
	await page.getByRole('button', { name: 'Register' }).click();
	await expect(page).toHaveURL('/');

	await page.getByRole('button', { name: 'New Chat', exact: true }).click();
	await expect(page).toHaveURL(/\/chat\/[0-9a-f-]+/);

	return email;
}

test('sending a message streams a Delta event before the final TurnCompleted event, then the fake assistant reply renders', async ({
	page,
}) => {
	// Registered before navigating to the chat page (rather than after registerAndStartChat
	// resolves) because the chat page's onMount opens its WebSocket as soon as it mounts —
	// Playwright's 'websocket' page event fires once, at creation time, so a listener attached
	// any later can silently miss the socket entirely and see no frames at all.
	const frameKinds: string[] = [];
	page.on('websocket', (ws) => {
		if (!ws.url().includes('/ws')) return;
		ws.on('framereceived', (frame) => {
			if (typeof frame.payload !== 'string') return;
			try {
				frameKinds.push(JSON.parse(frame.payload).kind);
			} catch {
				// not JSON; ignore
			}
		});
	});

	await registerAndStartChat(page);

	await page.getByPlaceholder('Ask me anything...').fill('hello there');
	await page.getByRole('button', { name: 'Send' }).click();

	await expect(page.getByRole('main').getByText('hello there')).toBeVisible();
	await expect(
		page.getByRole('main').getByText('This is a fake response for local development and testing.'),
	).toBeVisible();

	const deltaIndex = frameKinds.indexOf('Delta');
	const completedIndex = frameKinds.indexOf('TurnCompleted');
	expect(deltaIndex, `expected a Delta frame, got kinds: ${frameKinds.join(', ')}`).toBeGreaterThanOrEqual(0);
	expect(completedIndex, `expected TurnCompleted after Delta, got kinds: ${frameKinds.join(', ')}`).toBeGreaterThan(
		deltaIndex,
	);
});

test('a session history persists across a reload', async ({ page }) => {
	await registerAndStartChat(page);

	await page.getByPlaceholder('Ask me anything...').fill('remember this');
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(
		page.getByRole('main').getByText('This is a fake response for local development and testing.'),
	).toBeVisible();

	await page.reload();
	await expect(page.getByRole('main').getByText('remember this')).toBeVisible();
	await expect(
		page.getByRole('main').getByText('This is a fake response for local development and testing.'),
	).toBeVisible();
});

test('visiting a nonexistent session redirects to the empty state', async ({ page }) => {
	await registerAndStartChat(page);

	await page.goto('/chat/00000000-0000-0000-0000-000000000000');
	await expect(page).toHaveURL('/');
});

test('a turn failure shows an inline error without losing the sent text', async ({ page }) => {
	await registerAndStartChat(page);

	await page.getByPlaceholder('Ask me anything...').fill('__SIMULATE_TURN_FAILURE__');
	await page.getByRole('button', { name: 'Send' }).click();

	await expect(page.getByRole('main').getByText('__SIMULATE_TURN_FAILURE__')).toBeVisible();
	await expect(page.getByText(/something went wrong/i)).toBeVisible();
});

test('a second turn on the same page still renders live, without reconnecting', async ({ page }) => {
	await registerAndStartChat(page);

	await page.getByPlaceholder('Ask me anything...').fill('first message');
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(
		page.getByRole('main').getByText('This is a fake response for local development and testing.'),
	).toBeVisible();

	await page.getByPlaceholder('Ask me anything...').fill('second message');
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(page.getByRole('main').getByText('second message')).toBeVisible();
	await expect(
		page.getByRole('main').getByText('This is a fake response for local development and testing.'),
	).toHaveCount(2);
});
