import { afterEach, describe, expect, it, vi } from 'vitest';
import { render } from 'vitest-browser-svelte';
import { page, userEvent } from 'vitest/browser';
import { ImageOff, Lock } from '@lucide/svelte';
import ComposerFile from './ComposerFile.svelte';
import ModelFix from './ModelFix.svelte';
import VisionModelMenu from './VisionModelMenu.svelte';
import SentFiles from './SentFiles.svelte';
import FileNote from './FileNote.svelte';
import FileViewer from './FileViewer.svelte';
import PhotoViewer from './PhotoViewer.svelte';
import FileSheetHeader from './FileSheetHeader.svelte';
import ChatComposer from './ChatComposer.svelte';
import PhoneComposer from './PhoneComposer.svelte';
import { tokenBackground, tokenColor } from '../test-utils';

// Files in a chat (UX contract audit-chat-files, UXP-001): the composer's chips when an upload
// fails, a file is refused or the model can't see it (G3, G4, G6), the menu of models that can,
// the files of a sent message and of a shared link (G16), and the viewer (G12).

afterEach(() => {
	vi.restoreAllMocks();
	document.body.innerHTML = '';
});

const PIXEL = 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==';

describe('ComposerFile', () => {
	it('a failed upload says why in danger, with Retry and Remove, padded [6,8,6,10] in the danger stroke', async () => {
		const onretry = vi.fn();
		const onremove = vi.fn();
		await render(ComposerFile, { name: 'bca-oct-2026.pdf', status: 'failed', detail: 'Upload failed · connection lost', onretry, onremove });
		const chip = page.getByRole('group', { name: 'bca-oct-2026.pdf' }).element() as HTMLElement;
		const box = chip.firstElementChild as HTMLElement;
		const style = getComputedStyle(box);
		expect([style.paddingTop, style.paddingRight, style.paddingBottom, style.paddingLeft]).toEqual(['6px', '8px', '6px', '10px']);
		expect(style.backgroundColor).toBe(tokenBackground('nm-danger-bg'));
		expect(style.outlineColor).toBe(tokenColor('nm-danger-border'));
		expect(getComputedStyle(page.getByText('Upload failed · connection lost').element()).color).toBe(tokenColor('nm-danger'));
		await page.getByRole('button', { name: 'Retry bca-oct-2026.pdf' }).click();
		expect(onretry).toHaveBeenCalledOnce();
		await page.getByRole('button', { name: 'Remove bca-oct-2026.pdf' }).click();
		expect(onremove).toHaveBeenCalledOnce();
	});

	it('on a phone a failed chip offers its sheet (⋯) in place of the actions', async () => {
		const onmore = vi.fn();
		await render(ComposerFile, { name: 'bca-oct-2026.pdf', status: 'failed', detail: 'Upload failed · connection lost', phone: true, onretry: () => {}, onremove: () => {}, onmore });
		await expect.element(page.getByRole('button', { name: 'Retry bca-oct-2026.pdf' })).not.toBeInTheDocument();
		await page.getByRole('button', { name: 'Retry or remove bca-oct-2026.pdf' }).click();
		expect(onmore).toHaveBeenCalledOnce();
	});

	it('a refused file has Remove only, and Backspace on the focused chip removes it', async () => {
		const onremove = vi.fn();
		await render(ComposerFile, { name: 'kasir-setup.dmg', status: 'rejected', detail: '.dmg files can’t be read', onremove });
		await expect.element(page.getByRole('button', { name: /^Retry/ })).not.toBeInTheDocument();
		(page.getByRole('group', { name: 'kasir-setup.dmg' }).element() as HTMLElement).focus();
		await userEvent.keyboard('{Backspace}');
		expect(onremove).toHaveBeenCalledOnce();
	});

	it('a picture the model can’t see shows its thumb and the warning in the warning colour', async () => {
		await render(ComposerFile, { name: 'struk-belanja-nov.jpg', status: 'warning', detail: 'deepseek-v3 can’t see images', thumb: PIXEL, onremove: () => {} });
		const box = (page.getByRole('group', { name: 'struk-belanja-nov.jpg' }).element() as HTMLElement).firstElementChild as HTMLElement;
		expect(getComputedStyle(box).backgroundColor).toBe(tokenBackground('nm-warning-bg'));
		expect(getComputedStyle(page.getByText('deepseek-v3 can’t see images').element()).color).toBe(tokenColor('nm-warning'));
		expect(box.querySelector('img')!.getBoundingClientRect().width).toBe(28);
	});

	it('uploading and ready are the design system’s own chips', async () => {
		await render(ComposerFile, { name: 'bca-oct-2026.pdf', status: 'uploading', detail: 'Uploading again · 0% · 0 of 5 MB', progress: 0, onremove: () => {} });
		await expect.element(page.getByRole('progressbar', { name: 'bca-oct-2026.pdf' })).toHaveAttribute('aria-valuenow', '0');
		await expect.element(page.getByText('Uploading again · 0% · 0 of 5 MB')).toBeVisible();
	});
});

describe('ModelFix', () => {
	it('says what to do, and takes focus on Switch model as it appears', async () => {
		const onswitch = vi.fn();
		const ontextonly = vi.fn();
		await render(ModelFix, { onswitch, ontextonly, focus: true });
		await expect.element(page.getByText('Switch to a model that can see images, or send your message without the photo.')).toBeVisible();
		await expect.element(page.getByRole('button', { name: 'Switch model' })).toHaveFocus();
		await page.getByRole('button', { name: 'Send text only' }).click();
		expect(ontextonly).toHaveBeenCalledOnce();
	});

	it('on a phone drops “your message”', async () => {
		await render(ModelFix, { onswitch: () => {}, ontextonly: () => {}, phone: true });
		await expect.element(page.getByText('Switch to a model that can see images, or send without the photo.')).toBeVisible();
	});
});

describe('VisionModelMenu', () => {
	const groups = [
		{ group: 'Your keys', models: [{ key: 'k1', label: 'claude-sonnet', meta: 'Anthropic · reads images' }] },
		{ group: 'Nomi credits', models: [{ key: 'n1', label: 'gemini-flash', meta: 'Nomi credits · reads images' }] },
	];

	it('lists the models that can see, counts the hidden ones, focuses the first and moves with the arrows', async () => {
		const onpick = vi.fn();
		const onclose = vi.fn();
		await render(VisionModelMenu, { groups, hidden: 'deepseek-v3 hidden · can’t see images', settingsHref: '/settings/models', onpick, ontextonly: () => {}, onclose });
		const first = page.getByRole('menuitem', { name: /claude-sonnet/ });
		await expect.element(first).toHaveFocus();
		await expect.element(page.getByText('deepseek-v3 hidden · can’t see images')).toBeVisible();
		await expect.element(page.getByRole('menuitem', { name: 'Models & providers' })).toHaveAttribute('href', '/settings/models');
		await userEvent.keyboard('{ArrowDown}');
		await expect.element(page.getByRole('menuitem', { name: /gemini-flash/ })).toHaveFocus();
		await userEvent.keyboard('{Enter}');
		expect(onpick).toHaveBeenCalledWith('n1');
		await userEvent.keyboard('{Escape}');
		expect(onclose).toHaveBeenCalled();
	});

	it('with no model that can see, says so, focuses Add one, and keeps Send text only', async () => {
		const ontextonly = vi.fn();
		await render(VisionModelMenu, { groups: [], hidden: 'deepseek-v3 and 2 others hidden · can’t see images', settingsHref: '/settings/models', onpick: () => {}, ontextonly, onclose: () => {} });
		await expect.element(page.getByText('No model here can see images')).toBeVisible();
		await expect.element(page.getByRole('menuitem', { name: 'Add one in Models & providers' })).toHaveFocus();
		await page.getByRole('menuitem', { name: 'Send text only' }).click();
		expect(ontextonly).toHaveBeenCalledOnce();
	});

	it('on a phone is the sheet “Switch model · Models that can see images”', async () => {
		await render(VisionModelMenu, { groups, hidden: 'deepseek-v3 hidden · can’t see images', settingsHref: '/settings/models', onpick: () => {}, ontextonly: () => {}, onclose: () => {}, phone: true });
		const sheet = page.getByRole('dialog', { name: 'Switch model' });
		await expect.element(sheet).toBeVisible();
		await expect.element(sheet.getByText('Models that can see images')).toBeVisible();
		await expect.element(sheet.getByRole('button', { name: /gemini-flash/ })).toBeVisible();
	});
});

describe('FileSheetHeader', () => {
	it('names the file in mono and the reason in danger', async () => {
		const onclose = vi.fn();
		await render(FileSheetHeader, { name: 'bca-oct-2026.pdf', reason: 'Upload failed · connection lost', onclose });
		const title = page.getByRole('heading', { name: 'bca-oct-2026.pdf' }).element();
		expect(getComputedStyle(title).fontFamily).toContain('Geist Mono');
		expect(getComputedStyle(page.getByText('Upload failed · connection lost').element()).color).toBe(tokenColor('nm-danger'));
		await page.getByRole('button', { name: 'Close' }).click();
		expect(onclose).toHaveBeenCalledOnce();
	});
});

describe('SentFiles', () => {
	it('lists a message’s files, 200 wide, each opening when it can', async () => {
		const onopen = vi.fn();
		await render(SentFiles, { files: [{ name: 'bca-sep-2026.pdf', meta: 'PDF · 4,2 MB', onopen }, { name: 'struk.jpg', meta: 'JPG · 70 B', image: PIXEL, onopen: () => {} }] });
		const file = page.getByRole('button', { name: 'Open bca-sep-2026.pdf' });
		expect(file.element().getBoundingClientRect().width).toBe(200);
		await file.click();
		expect(onopen).toHaveBeenCalledOnce();
		await expect.element(page.getByRole('button', { name: 'Open struk.jpg' })).toBeVisible();
	});

	it('a private file is its lock and name only, not a button', async () => {
		await render(SentFiles, { files: [{ name: 'bca-sep-2026.pdf', meta: 'PDF · private', locked: true }] });
		await expect.element(page.getByText('PDF · private')).toBeVisible();
		await expect.element(page.getByRole('button')).not.toBeInTheDocument();
	});
});

describe('FileNote', () => {
	it('is a quiet line in mono, with its icon', async () => {
		await render(FileNote, { icon: ImageOff, text: 'Sent without struk-belanja-nov.jpg · deepseek-v3 can’t see images', mono: true });
		const text = page.getByText(/^Sent without/).element();
		expect(getComputedStyle(text).fontFamily).toContain('Geist Mono');
		expect(getComputedStyle(text).color).toBe(tokenColor('nm-text-3'));
	});

	it('or in sans, for a sentence', async () => {
		await render(FileNote, { icon: Lock, text: 'M. kept the files in this reply private. You see their names only.' });
		expect(getComputedStyle(page.getByText(/^M\. kept/).element()).fontFamily).not.toContain('Mono');
	});
});

describe('FileViewer', () => {
	it('opening: the real step and time over a page-shaped skeleton, focus on Close', async () => {
		await render(FileViewer, { name: 'bca-sep-2026.pdf', meta: 'PDF · 12 pages · your attachment', status: 'loading', step: 'Opening page 1 of 12', elapsed: '0:02', note: 'The answer stays usable while this opens.', downloadHref: '/attachments/1/content', onclose: () => {} });
		const panel = page.getByRole('dialog', { name: 'bca-sep-2026.pdf' });
		expect(panel.element().getBoundingClientRect().width).toBe(440);
		await expect.element(page.getByText('Opening page 1 of 12')).toBeVisible();
		await expect.element(page.getByText('· 0:02')).toBeVisible();
		await expect.element(page.getByRole('button', { name: 'Close' })).toHaveFocus();
		await expect.element(page.getByRole('link', { name: 'Download bca-sep-2026.pdf' })).toHaveAttribute('href', '/attachments/1/content');
	});

	it('the error says what happened, its code, and offers Try again and Download; Escape closes', async () => {
		const onretry = vi.fn();
		const onclose = vi.fn();
		await render(FileViewer, {
			name: 'bca-sep-2026.pdf',
			meta: 'PDF · 12 pages · your attachment',
			status: 'error',
			error: { body: 'Your file is fine and still in this chat. Try again, or download it and open page 1 yourself.', code: 'preview_timeout · 30 s' },
			downloadHref: '/attachments/1/content',
			onretry,
			onclose,
		});
		await expect.element(page.getByText('The preview didn’t load')).toBeVisible();
		await expect.element(page.getByText('preview_timeout · 30 s')).toBeVisible();
		await page.getByRole('button', { name: 'Try again' }).click();
		expect(onretry).toHaveBeenCalledOnce();
		await userEvent.keyboard('{Escape}');
		expect(onclose).toHaveBeenCalled();
	});

	it('on a phone is the whole screen', async () => {
		await render(FileViewer, { name: 'bca-sep-2026.pdf', meta: 'PDF · 12 pages · yours', status: 'loading', step: 'Opening page 1 of 12', elapsed: '0:01', downloadHref: '/x', onclose: () => {}, phone: true });
		expect(page.getByRole('dialog', { name: 'bca-sep-2026.pdf' }).element().getBoundingClientRect().width).toBe(window.innerWidth);
	});
});

describe('PhotoViewer', () => {
	it('zooms, downloads and closes on Escape, saying so', async () => {
		const onclose = vi.fn();
		await render(PhotoViewer, { name: 'struk.jpg', meta: 'JPG · 1,8 MB · your attachment', src: PIXEL, where: '1 of 1 photo in this message', downloadHref: '/attachments/2/content', onclose });
		await expect.element(page.getByRole('button', { name: 'Close' })).toHaveFocus();
		await expect.element(page.getByText('100%')).toBeVisible();
		await page.getByRole('button', { name: 'Zoom in' }).click();
		await expect.element(page.getByText('125%')).toBeVisible();
		await expect.element(page.getByText('esc closes')).toBeVisible();
		await userEvent.keyboard('{Escape}');
		expect(onclose).toHaveBeenCalled();
	});
});

describe('ChatComposer with files', () => {
	it('blocked: Send is off with the reason beside it, and Enter doesn’t send', async () => {
		const onsend = vi.fn();
		await render(ChatComposer, { placeholder: 'Ask anything…', onsend, blocked: '1 file didn’t upload. Retry or remove it to send.', value: 'What did I spend?' });
		const send = page.getByRole('button', { name: 'Send' });
		await expect.element(send).toBeDisabled();
		await expect.element(send).toHaveAttribute('aria-disabled', 'true');
		const hint = page.getByText('1 file didn’t upload. Retry or remove it to send.');
		await expect.element(hint).toBeVisible();
		expect(send.element().getAttribute('aria-describedby')).toBe(hint.element().id);
		await page.getByRole('textbox', { name: 'Message' }).click();
		await userEvent.keyboard('{Enter}');
		expect(onsend).not.toHaveBeenCalled();
	});

	it('sends files alone, without text', async () => {
		const onsend = vi.fn();
		await render(ChatComposer, { placeholder: 'Ask anything…', onsend, attached: true });
		await page.getByRole('button', { name: 'Send' }).click();
		expect(onsend).toHaveBeenCalledWith('');
	});
});

describe('PhoneComposer with files', () => {
	it('blocked: the notice line says why and Send shows off', async () => {
		const onsend = vi.fn();
		await render(PhoneComposer, { placeholder: 'Message', onsend, blocked: '1 file didn’t upload. Retry or remove it to send.', value: 'Hi' });
		await expect.element(page.getByText('1 file didn’t upload. Retry or remove it to send.')).toBeVisible();
		await expect.element(page.getByRole('button', { name: 'Send' })).toBeDisabled();
	});
});
