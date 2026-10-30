import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import MessageUser from '../conversation/MessageUser.svelte';
import VersionSwitcher from './VersionSwitcher.svelte';
import MessageEditor from './MessageEditor.svelte';
import RegenerateMenuFixture from './RegenerateMenuFixture.svelte';
import AttachMenuFixture from './AttachMenuFixture.svelte';
import CopiedNote from './CopiedNote.svelte';
import StopHint from './StopHint.svelte';
import AttachmentChip from './AttachmentChip.svelte';
import AttachmentImage from './AttachmentImage.svelte';
import VoiceRecorder from './VoiceRecorder.svelte';
import VoiceTranscribed from './VoiceTranscribed.svelte';
import ComposerLimit from './ComposerLimit.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Chat responses → Message controls & composer: every message can be stopped, edited, regenerated
// or copied. Edits branch instead of overwriting. Attachments and voice show progress and limits
// before you send.

afterEach(() => {
	vi.restoreAllMocks();
});

describe('VersionSwitcher', () => {
	it('steps between versions, each end disabled', async () => {
		const onprev = vi.fn();
		const onnext = vi.fn();
		await render(VersionSwitcher, { current: 2, total: 2, onprev, onnext });
		await expect.element(page.getByText('2 / 2')).toBeVisible();
		await expect.element(page.getByRole('button', { name: 'Next version' })).toBeDisabled();
		await page.getByRole('button', { name: 'Previous version' }).click();
		expect(onprev).toHaveBeenCalledOnce();
		expect(onnext).not.toHaveBeenCalled();
	});
});

describe('MessageUser controls', () => {
	it('offers copy and edit with the time, shown on hover', async () => {
		const write = vi.spyOn(navigator.clipboard, 'writeText').mockResolvedValue();
		const onedit = vi.fn();
		await render(MessageUser, { text: 'How much did I spend on food in October?', time: '14:02', onedit });
		const copy = page.getByRole('button', { name: 'Copy message' });
		expect(getComputedStyle(copy.element().parentElement!).opacity).toBe('0');
		await userEvent.hover(page.getByText('How much did I spend on food in October?'));
		expect(getComputedStyle(copy.element().parentElement!).opacity).toBe('1');
		await expect.element(page.getByText('14:02')).toBeVisible();
		expect(copy.element().querySelector('svg')?.getAttribute('width')).toBe('13');
		await copy.click();
		expect(write).toHaveBeenCalledWith('How much did I spend on food in October?');
		await page.getByRole('button', { name: 'Edit message' }).click();
		expect(onedit).toHaveBeenCalledOnce();
	});

	it('says it was edited and switches between versions', async () => {
		const onprev = vi.fn();
		await render(MessageUser, { text: 'Without office lunches?', time: '14:05', edited: true, version: { current: 2, total: 2, onprev, onnext: () => {} } });
		await expect.element(page.getByText('edited · 14:05')).toBeVisible();
		await page.getByRole('button', { name: 'Previous version' }).click();
		expect(onprev).toHaveBeenCalledOnce();
	});

	it('shows the files sent with it above the bubble', async () => {
		await render(MessageUser, {
			text: 'What did I spend on rent and food these two months?',
			files: [
				{ name: 'bca-sep-2026.pdf', detail: 'PDF · 12 pages' },
				{ name: 'bca-oct-2026.pdf', detail: 'PDF · 11 pages' },
			],
			images: [{ src: 'data:image/gif;base64,R0lGODlhAQABAAAAACw=', alt: 'Receipt photo' }],
		});
		expect(page.getByRole('listitem').elements()).toHaveLength(3);
		expect(getComputedStyle(page.getByText('bca-sep-2026.pdf').element()).fontFamily).toContain('Geist Mono');
		await expect.element(page.getByRole('img', { name: 'Receipt photo' })).toBeVisible();
	});
});

describe('MessageEditor', () => {
	it('edits in place and sends as a new version, never overwriting', async () => {
		const onsave = vi.fn();
		await render(MessageEditor, { value: 'How much did I spend on food in October?', version: '1 / 2', onsave, oncancel: () => {} });
		await expect.element(page.getByText('Sends as a new version. The old reply stays in 1 / 2.')).toBeVisible();
		const field = page.getByRole('textbox', { name: 'Edit message' });
		await expect.element(field).toHaveFocus();
		const save = page.getByRole('button', { name: 'Save & send' });
		await expect.element(save).toBeDisabled();
		await field.fill('How much did I spend on food in October, without office lunches?');
		await save.click();
		expect(onsave).toHaveBeenCalledWith('How much did I spend on food in October, without office lunches?');
	});

	it('cancels with Escape', async () => {
		const oncancel = vi.fn();
		await render(MessageEditor, { value: 'Hi', onsave: () => {}, oncancel });
		await userEvent.keyboard('{Escape}');
		expect(oncancel).toHaveBeenCalledOnce();
	});
});

describe('Regenerate, copy and stop', () => {
	it('regenerates the same way, shorter, longer, as a list, or with another model', async () => {
		const onpick = vi.fn();
		await render(RegenerateMenuFixture, { onpick });
		await page.getByRole('button', { name: 'Regenerate' }).click();
		await expect.element(page.getByText('⌘R')).toBeVisible();
		const group = page.getByRole('group', { name: 'With another model' });
		await expect.element(group).toBeVisible();
		expect(getComputedStyle(page.getByText('your key').element()).fontFamily).toContain('Geist Mono');
		await page.getByRole('menuitem', { name: /Shorter/ }).click();
		expect(onpick).toHaveBeenLastCalledWith({ style: 'shorter' });
		await page.getByRole('button', { name: 'Regenerate' }).click();
		await page.getByRole('menuitem', { name: /claude-sonnet/ }).click();
		expect(onpick).toHaveBeenLastCalledWith({ model: 'claude-sonnet' });
	});

	it('confirms a copy in an inverse note', async () => {
		await render(CopiedNote, { label: 'Copied as Markdown' });
		const note = page.getByRole('status');
		await expect.element(note).toHaveTextContent('Copied as Markdown');
		expect(getComputedStyle(note.element()).backgroundColor).toBe(tokenBackground('nm-text'));
	});

	it('stops the reply, with its key', async () => {
		const onstop = vi.fn();
		await render(StopHint, { onstop });
		const button = page.getByRole('button', { name: /Stop reply/ });
		await expect.element(button).toHaveAttribute('aria-keyshortcuts', 'Escape');
		await button.click();
		expect(onstop).toHaveBeenCalledOnce();
	});
});

describe('Attachments', () => {
	it('offers what you can attach and the limits', async () => {
		const onpick = vi.fn();
		await render(AttachMenuFixture, { onpick });
		await page.getByRole('button', { name: 'Attach' }).click();
		await expect.element(page.getByText('PDF, images, sheets, zip · max 20 MB')).toBeVisible();
		await expect.element(page.getByRole('menuitem', { name: /From Google Drive/ })).toHaveTextContent('Tara');
		await page.getByRole('menuitem', { name: /Paste a link/ }).click();
		expect(onpick).toHaveBeenCalledWith('link');
	});

	it('shows real upload progress, and cancels it', async () => {
		const onremove = vi.fn();
		await render(AttachmentChip, { name: 'bca-oct-2026.pdf', progress: 0.62, detail: '62% · 3,1 of 5 MB', onremove });
		const bar = page.getByRole('progressbar', { name: 'bca-oct-2026.pdf' });
		await expect.element(bar).toHaveAttribute('aria-valuenow', '62');
		const fill = bar.element().firstElementChild as HTMLElement;
		expect(getComputedStyle(fill).backgroundColor).toBe(tokenBackground('nm-accent'));
		await page.getByRole('button', { name: 'Cancel upload of bca-oct-2026.pdf' }).click();
		expect(onremove).toHaveBeenCalledOnce();
	});

	it('removes a ready file', async () => {
		const onremove = vi.fn();
		await render(AttachmentChip, { name: 'bca-sep-2026.pdf', detail: 'PDF · 4,2 MB', onremove });
		expect(page.getByRole('progressbar').elements()).toHaveLength(0);
		await page.getByRole('button', { name: 'Remove bca-sep-2026.pdf' }).click();
		expect(onremove).toHaveBeenCalledOnce();
	});
});

describe('AttachmentImage', () => {
	it('shows an attached picture as a small square, removable', async () => {
		const onremove = vi.fn();
		await render(AttachmentImage, { src: 'data:image/gif;base64,R0lGODlhAQABAAAAACw=', alt: 'Receipt photo', onremove });
		const image = page.getByRole('img', { name: 'Receipt photo' });
		expect(image.element().getBoundingClientRect().width).toBe(44);
		await page.getByRole('button', { name: 'Remove Receipt photo' }).click();
		expect(onremove).toHaveBeenCalledOnce();
	});
});

describe('Voice and limits', () => {
	it('records with a live level, the time and the language, and can be discarded or used', async () => {
		const oncancel = vi.fn();
		const ondone = vi.fn();
		const { container } = await render(VoiceRecorder, { elapsed: '00:14', levels: [0.2, 0.6, 1, 0.4], fading: 1, language: 'Bahasa Indonesia', oncancel, ondone });
		await expect.element(page.getByRole('group', { name: 'Recording' })).toBeVisible();
		expect(getComputedStyle(page.getByText('00:14').element()).fontFamily).toContain('Geist Mono');
		const bars = [...container.querySelectorAll('[data-bar]')] as HTMLElement[];
		expect(bars).toHaveLength(4);
		expect(getComputedStyle(bars[0]).backgroundColor).toBe(tokenBackground('nm-text-2'));
		expect(getComputedStyle(bars[3]).backgroundColor).toBe(tokenBackground('nm-border-strong'));
		expect(bars[2].getBoundingClientRect().height).toBeGreaterThan(bars[0].getBoundingClientRect().height);
		await page.getByRole('button', { name: 'Discard recording' }).click();
		await page.getByRole('button', { name: 'Use recording' }).click();
		expect(oncancel).toHaveBeenCalledOnce();
		expect(ondone).toHaveBeenCalledOnce();
	});

	it('notes the text came from audio', async () => {
		await render(VoiceTranscribed, { duration: '0:14' });
		await expect.element(page.getByText('transcribed from 0:14 of audio · edit before sending')).toBeVisible();
	});

	it('says a message is too long, by how much, and what to do', async () => {
		const onfile = vi.fn();
		await render(ComposerLimit, { count: '12.480', max: '10.000', onfile });
		expect(getComputedStyle(page.getByText('12.480 / 10.000').element()).color).toBe(tokenColor('nm-danger'));
		await expect.element(page.getByText('Too long for one message.')).toBeVisible();
		await page.getByRole('button', { name: 'Send as a file instead' }).click();
		expect(onfile).toHaveBeenCalledOnce();
	});
});

