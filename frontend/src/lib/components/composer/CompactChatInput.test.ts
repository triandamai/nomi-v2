// @vitest-environment jsdom
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { createRawSnippet, flushSync, mount, tick, unmount } from 'svelte';
import CompactChatInput from './CompactChatInput.svelte';
import type { ComposerFile } from '$lib/attachments';
import type { ThinkingLevel } from '$lib/types';
import type { MentionOption } from '$lib/components/ds/chat/mention';

// jsdom has no popover API or ResizeObserver; the menus only need them to exist.
beforeAll(() => {
	globalThis.ResizeObserver ??= class {
		observe() {}
		unobserve() {}
		disconnect() {}
	};
	HTMLElement.prototype.showPopover ??= function () {};
	HTMLElement.prototype.hidePopover ??= function () {};
});

type Payload = { text: string; thinkingLevel: ThinkingLevel | null; attachments: ComposerFile[] };

interface Props {
	placeholder?: string;
	value?: string;
	thinkingLevel?: ThinkingLevel | null;
	onthinkingchange?: (level: ThinkingLevel) => void;
	attachments?: ComposerFile[];
	onsubmit?: (payload: Payload) => void;
	onfiles?: (files: File[]) => void;
	onrecord?: () => void;
	mentionOptions?: MentionOption[];
	transcribed?: string | null;
	disabled?: boolean;
	busy?: boolean;
	withModel?: boolean;
}

let component: ReturnType<typeof mount> | null = null;

function render({ withModel = false, onsubmit = () => {}, ...props }: Props = {}) {
	const target = document.createElement('div');
	document.body.append(target);
	const modelControl = withModel
		? createRawSnippet(() => ({ render: () => '<button type="button" data-testid="model">Claude Sonnet</button>' }))
		: undefined;
	component = mount(CompactChatInput, { target, props: { onsubmit, modelControl, ...props } });
	flushSync();
	return target;
}

function setPlatform(platform: string) {
	Object.defineProperty(navigator, 'platform', { value: platform, configurable: true });
}

function setCoarsePointer(coarse: boolean) {
	window.matchMedia = ((query: string) =>
		({ matches: coarse && query.includes('coarse'), media: query, addEventListener() {}, removeEventListener() {} }) as unknown as MediaQueryList) as typeof window.matchMedia;
}

afterEach(() => {
	if (component) unmount(component);
	component = null;
	document.body.innerHTML = '';
	setPlatform('');
	setCoarsePointer(false);
});

const field = (root: ParentNode) => root.querySelector('textarea')!;
const send = (root: ParentNode) => root.querySelector<HTMLButtonElement>('.nomi-send')!;
const fileInput = (root: ParentNode) => root.querySelector<HTMLInputElement>('input[type="file"]')!;
const photoInput = (root: ParentNode) => root.querySelector<HTMLInputElement>('input[type="file"][accept="image/*"]')!;
const plus = (root: ParentNode) => root.querySelector<HTMLButtonElement>('button[aria-label="Attach a file"]')!;

/** Opens the [+] menu (in the top layer, outside `root`) and gives its items. */
async function plusItems(root: ParentNode) {
	plus(root).click();
	await tick();
	flushSync();
	return [...document.querySelectorAll<HTMLElement>('[role="menuitem"]')];
}

function type(root: ParentNode, text: string) {
	const input = field(root);
	input.value = text;
	input.dispatchEvent(new Event('input', { bubbles: true }));
	flushSync();
}

function key(root: ParentNode, init: KeyboardEventInit) {
	const event = new KeyboardEvent('keydown', { bubbles: true, cancelable: true, ...init });
	field(root).dispatchEvent(event);
	flushSync();
	return event;
}

const uploaded = (name: string): ComposerFile => ({
	key: name,
	name,
	size: 2048,
	kind: 'image',
	progress: 1,
	uploaded: { id: `id-${name}`, name, kind: 'image', mime: 'image/png', size: 2048, status: 'ready', reference: `<attachment id="id-${name}"/>` },
	thumb: null,
	controller: new AbortController(),
});

describe('CompactChatInput', () => {
	it('shows the placeholder it is given', () => {
		expect(field(render({ placeholder: 'Ask about this project…' })).placeholder).toBe('Ask about this project…');
	});

	it('falls back to "Ask anything…"', () => {
		expect(field(render()).placeholder).toBe('Ask anything…');
	});

	it('keeps Send disabled until there is text, then sends the text, thinking level and files', () => {
		const onsubmit = vi.fn();
		const root = render({ onsubmit, thinkingLevel: 'medium' });
		expect(send(root).disabled).toBe(true);
		type(root, '   ');
		expect(send(root).disabled).toBe(true);
		type(root, 'Plan my week');
		expect(send(root).disabled).toBe(false);
		send(root).click();
		flushSync();
		expect(onsubmit).toHaveBeenCalledWith({ text: 'Plan my week', thinkingLevel: 'medium', attachments: [] });
	});

	it('can send just a file', () => {
		const onsubmit = vi.fn();
		const photo = uploaded('photo.png');
		const root = render({ onsubmit, attachments: [photo] });
		expect(send(root).disabled).toBe(false);
		send(root).click();
		expect(onsubmit).toHaveBeenCalledWith({ text: '', thinkingLevel: null, attachments: [photo] });
	});

	it('holds Send while a file is still uploading', () => {
		const root = render({ value: 'Look', attachments: [{ ...uploaded('big.mov'), uploaded: null, progress: 0.4 }] });
		expect(send(root).disabled).toBe(true);
	});

	it('sends on Enter with a mouse and keyboard', () => {
		const onsubmit = vi.fn();
		const root = render({ onsubmit });
		type(root, 'Hello');
		const event = key(root, { key: 'Enter' });
		expect(onsubmit).toHaveBeenCalledTimes(1);
		expect(event.defaultPrevented).toBe(true);
	});

	it('keeps Shift+Enter for a new line', () => {
		const onsubmit = vi.fn();
		const root = render({ onsubmit });
		type(root, 'Hello');
		const event = key(root, { key: 'Enter', shiftKey: true });
		expect(onsubmit).not.toHaveBeenCalled();
		expect(event.defaultPrevented).toBe(false);
	});

	it('never sends while an input method is composing', () => {
		const onsubmit = vi.fn();
		const root = render({ onsubmit });
		type(root, 'にほん');
		const event = key(root, { key: 'Enter', isComposing: true });
		expect(onsubmit).not.toHaveBeenCalled();
		expect(event.defaultPrevented).toBe(false);
	});

	it('lets Enter make a new line on a touch screen, where Send sends', () => {
		setCoarsePointer(true);
		const onsubmit = vi.fn();
		const root = render({ onsubmit });
		type(root, 'Hello');
		const event = key(root, { key: 'Enter' });
		expect(onsubmit).not.toHaveBeenCalled();
		expect(event.defaultPrevented).toBe(false);
		send(root).click();
		expect(onsubmit).toHaveBeenCalledTimes(1);
	});

	it('does not send an empty message on Enter', () => {
		const onsubmit = vi.fn();
		const root = render({ onsubmit });
		key(root, { key: 'Enter' });
		expect(onsubmit).not.toHaveBeenCalled();
	});

	it('changes the thinking level it sends when one is picked', async () => {
		const onsubmit = vi.fn();
		const onthinkingchange = vi.fn();
		const root = render({ onsubmit, onthinkingchange, thinkingLevel: 'medium' });
		root.querySelector<HTMLButtonElement>('button[aria-haspopup="menu"][aria-label^="Thinking"]')!.click();
		flushSync();
		await tick();
		[...root.querySelectorAll<HTMLButtonElement>('[role="menuitemradio"]')].find((o) => o.textContent?.trim() === 'Max')!.click();
		flushSync();
		expect(onthinkingchange).toHaveBeenCalledWith('max');
		type(root, 'Think hard');
		send(root).click();
		expect(onsubmit).toHaveBeenCalledWith({ text: 'Think hard', thinkingLevel: 'max', attachments: [] });
	});

	it('hides the thinking picker when there is no level', () => {
		const root = render({ thinkingLevel: null });
		expect(root.querySelector('[role="menuitemradio"]')).toBeNull();
	});

	it('offers a file, a photo and a link under +, with the limit underneath', async () => {
		const items = await plusItems(render());
		expect(items.map((item) => item.textContent?.trim())).toEqual(['Upload a file', 'Photo or screenshot', 'Paste a link']);
		expect(document.body.textContent).toContain('max 10 MB');
	});

	it('names the file shortcut on [+], ⌘U on a Mac', () => {
		expect(plus(render()).title).toContain('Ctrl U');
		unmount(component!);
		setPlatform('MacIntel');
		expect(plus(render()).title).toContain('⌘U');
	});

	it('opens the file picker from "Upload a file", the photo picker from "Photo or screenshot"', async () => {
		const root = render();
		const files = vi.spyOn(fileInput(root), 'click').mockImplementation(() => {});
		const photos = vi.spyOn(photoInput(root), 'click').mockImplementation(() => {});
		(await plusItems(root))[0].click();
		(await plusItems(root))[1].click();
		expect(files).toHaveBeenCalledTimes(1);
		expect(photos).toHaveBeenCalledTimes(1);
	});

	it('puts a pasted link in the message, on its own line', async () => {
		const root = render({ value: 'Read this' });
		(await plusItems(root))[2].click();
		await tick();
		flushSync();
		const link = root.querySelector<HTMLInputElement>('input[type="url"]')!;
		link.value = 'https://example.com/a';
		link.dispatchEvent(new Event('input', { bubbles: true }));
		flushSync();
		link.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }));
		flushSync();
		expect(field(root).value).toBe('Read this\nhttps://example.com/a');
		expect(root.querySelector('input[type="url"]')).toBeNull();
	});

	it('records a voice note from the mic when it can, and says when the text came from one', () => {
		const onrecord = vi.fn();
		const root = render({ onrecord, value: 'Coffee 25k', transcribed: '0:14' });
		root.querySelector<HTMLButtonElement>('button[aria-label="Record a voice note"]')!.click();
		expect(onrecord).toHaveBeenCalledTimes(1);
		expect(root.textContent).toContain('transcribed from 0:14 of audio');
	});

	it('hands picked files over', () => {
		const onfiles = vi.fn();
		const root = render({ onfiles });
		const file = new File(['hi'], 'note.txt', { type: 'text/plain' });
		const input = fileInput(root);
		Object.defineProperty(input, 'files', { value: [file], configurable: true });
		input.dispatchEvent(new Event('change', { bubbles: true }));
		expect(onfiles).toHaveBeenCalledWith([file]);
	});

	it('opens the file picker on Ctrl+U', () => {
		const root = render();
		const click = vi.spyOn(fileInput(root), 'click').mockImplementation(() => {});
		const event = key(root, { key: 'u', ctrlKey: true });
		expect(click).toHaveBeenCalledTimes(1);
		expect(event.defaultPrevented).toBe(true);
	});

	it('opens the file picker on ⌘U on a Mac', () => {
		setPlatform('MacIntel');
		const root = render();
		const click = vi.spyOn(fileInput(root), 'click').mockImplementation(() => {});
		const event = key(root, { key: 'u', metaKey: true });
		expect(click).toHaveBeenCalledTimes(1);
		expect(event.defaultPrevented).toBe(true);
	});

	it('puts the model control in the toolbar', () => {
		const root = render({ withModel: true });
		const toolbar = root.querySelector('[data-composer-toolbar]')!;
		expect(toolbar.querySelector('[data-testid="model"]')).not.toBeNull();
		expect(toolbar.contains(plus(root))).toBe(true);
		expect(toolbar.contains(send(root))).toBe(true);
	});

	it('disables the field and Send when disabled', () => {
		const onsubmit = vi.fn();
		const root = render({ disabled: true, value: 'Hello', onsubmit });
		expect(field(root).disabled).toBe(true);
		expect(send(root).disabled).toBe(true);
		key(root, { key: 'Enter' });
		expect(onsubmit).not.toHaveBeenCalled();
	});

	it('does not send on Enter while the crew is busy', () => {
		const onsubmit = vi.fn();
		const root = render({ busy: true, value: 'Next thing', onsubmit });
		key(root, { key: 'Enter' });
		expect(onsubmit).not.toHaveBeenCalled();
	});

	describe('"@"', () => {
		const crew: MentionOption[] = [
			{ kind: 'crew', agent: 'dana', name: 'Dana', role: 'Money' },
			{ kind: 'crew', agent: 'koda', name: 'Koda', role: 'Code' },
			{ kind: 'crew', agent: 'kala', name: 'Kala', role: 'Reminders' },
		];
		const options = (root: ParentNode) => [...root.querySelectorAll('[role="option"]')].map((option) => option.textContent?.replace('↵', '').replace(/\s+/g, ' ').trim());

		it('lists the crew matching what follows it, and Enter picks one instead of sending', () => {
			const onsubmit = vi.fn();
			const root = render({ mentionOptions: crew, onsubmit });
			type(root, 'ask @k');
			expect(options(root)).toEqual(['Koda Code', 'Kala Reminders']);
			expect(field(root).getAttribute('aria-expanded')).toBe('true');
			key(root, { key: 'ArrowDown' });
			expect(field(root).getAttribute('aria-activedescendant')).toMatch(/-option-1$/);
			key(root, { key: 'Enter' });
			expect(onsubmit).not.toHaveBeenCalled();
			expect(field(root).value).toBe('ask @Kala ');
			expect(root.querySelector('[role="listbox"]')).toBeNull();
		});

		it('closes on Escape, and stays out of an email address', () => {
			const root = render({ mentionOptions: crew });
			type(root, '@d');
			key(root, { key: 'Escape' });
			expect(root.querySelector('[role="listbox"]')).toBeNull();
			type(root, 'mail dana@d');
			expect(root.querySelector('[role="listbox"]')).toBeNull();
		});
	});

	describe('the length limit', () => {
		it('holds Send over the limit and offers to send the text as a file', () => {
			const onfiles = vi.fn();
			const onsubmit = vi.fn();
			const root = render({ onfiles, onsubmit });
			type(root, 'a'.repeat(10_001));
			expect(root.querySelector('[role="alert"]')?.textContent).toContain('10,001 / 10,000');
			expect(send(root).disabled).toBe(true);
			key(root, { key: 'Enter' });
			expect(onsubmit).not.toHaveBeenCalled();
			[...root.querySelectorAll('button')].find((button) => button.textContent === 'Send as a file instead')!.click();
			flushSync();
			const [[files]] = onfiles.mock.calls as [[File[]]];
			expect(files[0].name).toBe('message.txt');
			expect(files[0].size).toBe(10_001);
			expect(field(root).value).toBe('');
		});
	});
});
