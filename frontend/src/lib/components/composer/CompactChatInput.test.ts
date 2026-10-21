// @vitest-environment jsdom
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { createRawSnippet, flushSync, mount, tick, unmount } from 'svelte';
import CompactChatInput from './CompactChatInput.svelte';
import type { ComposerFile } from '$lib/attachments';
import type { ThinkingLevel } from '$lib/types';

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
	onvoicenote?: () => void;
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
const plus = (root: ParentNode) => root.querySelector<HTMLButtonElement>('button[aria-label="Attach files or a voice note"]')!;
const plusItems = (root: ParentNode) => [...plus(root).closest('.m3-menu-anchor')!.querySelectorAll<HTMLButtonElement>('[role="menuitem"]')];

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

	it('offers exactly "Images and files" and "Record a voice note" under +', () => {
		const root = render({ onvoicenote: () => {} });
		const items = plusItems(root);
		expect(items.map((item) => item.querySelector('[data-label]')?.textContent?.trim())).toEqual(['Images and files', 'Record a voice note']);
		expect(items[0].querySelector('kbd')?.textContent?.trim()).toBe('Ctrl U');
		expect(items[1].querySelector('kbd')).toBeNull();
	});

	it('shows ⌘U on a Mac', () => {
		setPlatform('MacIntel');
		const root = render();
		expect(plusItems(root)[0].querySelector('kbd')?.textContent?.trim()).toBe('⌘U');
	});

	it('leaves voice notes out where they can’t be recorded', () => {
		const root = render();
		expect(plusItems(root).map((item) => item.querySelector('[data-label]')?.textContent?.trim())).toEqual(['Images and files']);
	});

	it('opens the file picker from "Images and files"', () => {
		const root = render();
		const click = vi.spyOn(fileInput(root), 'click').mockImplementation(() => {});
		plusItems(root)[0].click();
		expect(click).toHaveBeenCalledTimes(1);
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

	it('starts a voice note from "Record a voice note"', () => {
		const onvoicenote = vi.fn();
		const root = render({ onvoicenote });
		plusItems(root)[1].click();
		expect(onvoicenote).toHaveBeenCalledTimes(1);
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
});
