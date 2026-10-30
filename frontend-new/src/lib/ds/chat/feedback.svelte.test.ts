import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import ResponseActions from './ResponseActions.svelte';
import FeedbackSent from './FeedbackSent.svelte';
import DeletedReply from './DeletedReply.svelte';
import DislikeFeedback from './DislikeFeedback.svelte';
import SharePanel from './SharePanel.svelte';
import ShareDialog from './ShareDialog.svelte';
import DeleteReplyDialog from './DeleteReplyDialog.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Chat responses → Response feedback: every reply ends with the same quiet bar. Like is one tap;
// dislike asks why; share masks your numbers by default; delete confirms and can be undone.

afterEach(() => {
	vi.restoreAllMocks();
});

const preview = { agent: 'nomi' as const, name: 'Nomi', time: '14:02', excerpt: "Food is October's biggest jump: Rp ••• against Rp ••• in September." };

describe('ResponseActions', () => {
	it('names every action and shows the model, time and tokens in mono', async () => {
		await render(ResponseActions, { text: 'Food is the jump.', meta: 'deepseek-flash · 1.4s · 812 tokens', onlike: () => {}, ondislike: () => {}, onshare: () => {}, onregenerate: () => {}, ondelete: () => {} });
		for (const name of ['Copy', 'Good reply', 'Bad reply', 'Share', 'Regenerate', 'Delete reply']) {
			await expect.element(page.getByRole('button', { name, exact: true })).toBeVisible();
		}
		expect(getComputedStyle(page.getByText('deepseek-flash · 1.4s · 812 tokens').element()).fontFamily).toContain('Geist Mono');
	});

	it('marks a like in mint and a dislike in text colour, each a pressed toggle', async () => {
		const onlike = vi.fn();
		await render(ResponseActions, { text: 'x', feedback: 'liked', onlike, ondislike: () => {} });
		const like = page.getByRole('button', { name: 'Good reply' });
		await expect.element(like).toHaveAttribute('aria-pressed', 'true');
		await expect.element(page.getByRole('button', { name: 'Bad reply' })).toHaveAttribute('aria-pressed', 'false');
		expect(getComputedStyle(like.element()).backgroundColor).toBe(tokenBackground('nm-surface-2'));
		expect(getComputedStyle(like.element()).color).toBe(tokenColor('nm-accent'));
		await like.click();
		expect(onlike).toHaveBeenCalledOnce();
	});

	it('copies the reply and says so', async () => {
		const write = vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue();
		await render(ResponseActions, { text: 'Food is the jump.' });
		await page.getByRole('button', { name: 'Copy' }).click();
		expect(write).toHaveBeenCalledWith('Food is the jump.');
		await expect.element(page.getByRole('button', { name: 'Copied' })).toBeVisible();
	});
});

describe('After feedback or delete', () => {
	it('thanks you and says exactly what was sent', async () => {
		const onclick = vi.fn();
		await render(FeedbackSent, { retry: { label: 'Try again with Claude Sonnet', onclick } });
		await expect.element(page.getByText('Thanks. Feedback sent with this reply only.')).toBeVisible();
		await page.getByRole('button', { name: 'Try again with Claude Sonnet' }).click();
		expect(onclick).toHaveBeenCalledOnce();
	});

	it('says when the whole chat went with it', async () => {
		await render(FeedbackSent, { withChat: true });
		await expect.element(page.getByText('Thanks. Feedback sent with the whole chat.')).toBeVisible();
	});

	it('leaves a trace of a deleted reply, with undo and regenerate', async () => {
		const onundo = vi.fn();
		const onregenerate = vi.fn();
		await render(DeletedReply, { name: 'Nomi', onundo, onregenerate });
		expect(getComputedStyle(page.getByText("You deleted Nomi's reply").element()).fontStyle).toBe('italic');
		await page.getByRole('button', { name: 'Undo' }).click();
		await page.getByRole('button', { name: 'Regenerate' }).click();
		expect(onundo).toHaveBeenCalledOnce();
		expect(onregenerate).toHaveBeenCalledOnce();
	});
});

describe('DislikeFeedback', () => {
	it('asks what went wrong, then optional detail, the chat left out by default', async () => {
		const onsend = vi.fn();
		await render(DislikeFeedback, { onsend, oncancel: () => {} });
		await expect.element(page.getByText('What went wrong?')).toBeVisible();
		const send = page.getByRole('button', { name: 'Send feedback' });
		await expect.element(send).toBeDisabled();
		const accurate = page.getByRole('button', { name: 'Not accurate' });
		await accurate.click();
		await expect.element(accurate).toHaveAttribute('aria-pressed', 'true');
		expect(getComputedStyle(accurate.element()).backgroundColor).toBe(tokenBackground('nm-text'));
		await page.getByRole('button', { name: 'Wrong crew member' }).click();
		await page.getByRole('textbox', { name: 'What went wrong?' }).fill('It counted office lunches.');
		await expect.element(page.getByRole('checkbox', { name: 'Include the whole chat' })).not.toBeChecked();
		await send.click();
		expect(onsend).toHaveBeenCalledWith({ reasons: ['inaccurate', 'wrong_crew'], note: 'It counted office lunches.', includeChat: false });
	});

	it('closes without sending', async () => {
		const oncancel = vi.fn();
		await render(DislikeFeedback, { onsend: () => {}, oncancel });
		await page.getByRole('button', { name: 'Close' }).click();
		await page.getByRole('button', { name: 'Cancel' }).click();
		expect(oncancel).toHaveBeenCalledTimes(2);
	});
});

describe('Share', () => {
	it('masks amounts and names by default and shows the link in mono', async () => {
		await render(SharePanel, { preview, chatMessages: 6, url: 'https://nomi.app/s/k7Qe2-r4', expiry: 'in 7 days', onclose: () => {} });
		await expect.element(page.getByRole('radio', { name: 'This reply' })).toBeChecked();
		const scope = page.getByRole('radiogroup', { name: 'What to share' }).element();
		expect(getComputedStyle(scope).backgroundColor).toBe(tokenBackground('nm-bg'));
		expect(getComputedStyle(scope).borderTopColor).toBe(tokenColor('nm-border'));
		await expect.element(page.getByRole('radio', { name: /Whole chat/ })).toHaveTextContent('6 msgs');
		await expect.element(page.getByRole('switch', { name: 'Hide amounts and names' })).toHaveAttribute('aria-checked', 'true');
		await expect.element(page.getByRole('switch', { name: 'Anyone with the link can view' })).toHaveAttribute('aria-checked', 'true');
		await expect.element(page.getByText(/Rp ••• against Rp •••/)).toBeVisible();
		expect(getComputedStyle(page.getByText('nomi.app/s/k7Qe2-r4').element()).fontFamily).toContain('Geist Mono');
		await expect.element(page.getByRole('button', { name: /in 7 days/ })).toBeVisible();
		await expect.element(page.getByText('You can revoke this link any time in Settings › Data')).toBeVisible();
	});

	it('reports each change and copies the link', async () => {
		const write = vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue();
		const onchange = vi.fn();
		const oncopy = vi.fn();
		await render(SharePanel, { preview, chatMessages: 6, url: 'https://nomi.app/s/k7Qe2-r4', expiry: 'in 7 days', onchange, oncopy, onclose: () => {} });
		await page.getByRole('switch', { name: 'Hide amounts and names' }).click();
		expect(onchange).toHaveBeenLastCalledWith({ scope: 'reply', mask: false, anyone: true });
		await page.getByRole('radio', { name: /Whole chat/ }).click();
		expect(onchange).toHaveBeenLastCalledWith({ scope: 'chat', mask: false, anyone: true });
		await page.getByRole('button', { name: 'Copy link' }).click();
		expect(write).toHaveBeenCalledWith('https://nomi.app/s/k7Qe2-r4');
		expect(oncopy).toHaveBeenCalledOnce();
	});

	it('opens as a modal dialog named Share, and closes', async () => {
		const onclose = vi.fn();
		await render(ShareDialog, { open: true, preview, chatMessages: 6, url: 'https://nomi.app/s/k7Qe2-r4', expiry: 'in 7 days', onclose });
		const dialog = page.getByRole('dialog', { name: 'Share' });
		await expect.element(dialog).toBeVisible();
		await page.getByRole('button', { name: 'Close' }).click();
		await expect.element(dialog).not.toBeInTheDocument();
		expect(onclose).toHaveBeenCalledOnce();
	});
});

describe('DeleteReplyDialog', () => {
	it('says what leaves and what stays, then deletes', async () => {
		const ondelete = vi.fn();
		await render(DeleteReplyDialog, { open: true, name: 'Nomi', ondelete });
		await expect.element(page.getByRole('dialog', { name: 'Delete this reply?' })).toBeVisible();
		await expect.element(page.getByText(/Nomi's reply and its cards leave this chat/)).toBeVisible();
		await page.getByRole('button', { name: 'Delete reply' }).click();
		expect(ondelete).toHaveBeenCalledOnce();
		await expect.element(page.getByRole('dialog')).not.toBeInTheDocument();
	});

	it('cancels with Escape', async () => {
		const ondelete = vi.fn();
		await render(DeleteReplyDialog, { open: true, name: 'Nomi', ondelete });
		await userEvent.keyboard('{Escape}');
		await expect.element(page.getByRole('dialog')).not.toBeInTheDocument();
		expect(ondelete).not.toHaveBeenCalled();
	});
});
