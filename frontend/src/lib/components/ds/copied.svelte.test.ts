// @vitest-environment jsdom
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, unmount, type Component } from 'svelte';
import Composer from './inputs/Composer.svelte';
import DateDivider from './chat/DateDivider.svelte';
import NewDivider from './chat/NewDivider.svelte';
import LoadingOlder from './chat/LoadingOlder.svelte';
import JumpToLatest from './chat/JumpToLatest.svelte';
import Boundary from './chat/Boundary.svelte';
import BackgroundNote from './chat/BackgroundNote.svelte';
import FirstRun from './build/FirstRun.svelte';
import ResizeHandle from './build/ResizeHandle.svelte';
import PublishButton from './build/PublishButton.svelte';
import RequestQueue from './build/RequestQueue.svelte';
import PublishRequest from './build/PublishRequest.svelte';
import RoleMenu from './build/RoleMenu.svelte';
import BuildInvite from './build/BuildInvite.svelte';
import JoinedNote from './build/JoinedNote.svelte';
import FollowBar from './build/FollowBar.svelte';
import NavBar from './mobile/NavBar.svelte';
import MobileComposer from './mobile/Composer.svelte';
import UserMessage from './mobile/UserMessage.svelte';
import WorkspaceSwitch from './mobile/WorkspaceSwitch.svelte';
import AskBar from './mobile/AskBar.svelte';

// The twins copied from frontend-new as they are (thread chrome, the composer, build and mobile)
// behave like frontend-new's: the same roles, names and events. Their look comes from ds-twins.css,
// so nothing here checks styling.

beforeAll(() => {
	globalThis.ResizeObserver ??= class {
		observe() {}
		unobserve() {}
		disconnect() {}
	};
	HTMLElement.prototype.showPopover ??= function () {};
	HTMLElement.prototype.hidePopover ??= function () {};
});

let mounted: ReturnType<typeof mount>[] = [];

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function render(component: Component<any>, props: Record<string, unknown> = {}): HTMLElement {
	const target = document.body.appendChild(document.createElement('div'));
	mounted.push(mount(component, { target, props }));
	flushSync();
	return target;
}

afterEach(() => {
	for (const component of mounted) unmount(component);
	mounted = [];
	document.body.innerHTML = '';
});

/** The element with this role whose accessible name (its label, else its text) matches. */
function named<T extends HTMLElement = HTMLElement>(root: HTMLElement, selector: string, name: string | RegExp): T {
	const found = [...root.querySelectorAll<T>(selector)].find((element) => {
		const label = (element.getAttribute('aria-label') ?? element.textContent ?? '').replace(/\s+/g, ' ').trim();
		return typeof name === 'string' ? label === name : name.test(label);
	});
	if (!found) throw new Error(`no ${selector} named ${name}`);
	return found;
}

const button = (root: HTMLElement, name: string | RegExp) => named<HTMLButtonElement>(root, 'button', name);

function type(field: HTMLTextAreaElement | HTMLInputElement, text: string) {
	field.value = text;
	field.dispatchEvent(new Event('input', { bubbles: true }));
	flushSync();
}

function key(element: Element, name: string, init: KeyboardEventInit = {}) {
	element.dispatchEvent(new KeyboardEvent('keydown', { key: name, bubbles: true, cancelable: true, ...init }));
	flushSync();
}

const click = (element: HTMLElement) => {
	element.click();
	flushSync();
};

describe('Composer', () => {
	const props = { placeholder: 'Ask Nomi, or hand something to the crew…', agent: { id: 'nomi', name: 'Nomi' }, model: 'deepseek-flash' };

	it('only offers Send when there is something to send', () => {
		const root = render(Composer, props);
		const field = named<HTMLTextAreaElement>(root, 'textarea', 'Message');
		expect(field.placeholder).toBe(props.placeholder);
		expect(button(root, 'Send').disabled).toBe(true);
		type(field, '   ');
		expect(button(root, 'Send').disabled).toBe(true);
		type(field, 'hi');
		expect(button(root, 'Send').disabled).toBe(false);
	});

	it('sends with Enter, keeps Shift+Enter for a new line, and clears after sending', () => {
		const onsend = vi.fn();
		const root = render(Composer, { ...props, onsend });
		const field = named<HTMLTextAreaElement>(root, 'textarea', 'Message');
		type(field, 'Food spend in October');
		key(field, 'Enter', { shiftKey: true });
		expect(onsend).not.toHaveBeenCalled();
		key(field, 'Enter');
		expect(onsend).toHaveBeenCalledWith('Food spend in October');
		expect(field.value).toBe('');
	});

	it('does not send while an input method is composing a word', () => {
		const onsend = vi.fn();
		const root = render(Composer, { ...props, onsend });
		const field = named<HTMLTextAreaElement>(root, 'textarea', 'Message');
		type(field, 'にほん');
		key(field, 'Enter', { isComposing: true });
		expect(onsend).not.toHaveBeenCalled();
	});

	it('sends with the Send button too, and hands the chips’ clicks out', () => {
		const onsend = vi.fn();
		const onpickagent = vi.fn();
		const onattach = vi.fn();
		const root = render(Composer, { ...props, onsend, onpickagent, onattach });
		type(named<HTMLTextAreaElement>(root, 'textarea', 'Message'), 'hello');
		click(button(root, 'Send'));
		expect(onsend).toHaveBeenCalledWith('hello');
		click(button(root, /Nomi/));
		click(button(root, 'Attach a file'));
		expect(onpickagent).toHaveBeenCalledOnce();
		expect(onattach).toHaveBeenCalledOnce();
	});
});

describe('long threads', () => {
	it('jumps to the latest, saying how many replies are new', () => {
		const onclick = vi.fn();
		click(button(render(JumpToLatest, { count: 2, onclick }), '2 new replies'));
		expect(onclick).toHaveBeenCalledOnce();
		expect(button(render(JumpToLatest, { count: 0, onclick }), 'Jump to latest')).toBeTruthy();
	});

	it('marks what arrived while you were away, and divides days', () => {
		expect(render(NewDivider, { count: 2 }).querySelector('[role="separator"]')!.getAttribute('aria-label')).toBe('NEW · 2 replies while you were away');
		expect(render(DateDivider, { label: 'Yesterday' }).querySelector('[role="separator"]')!.getAttribute('aria-label')).toBe('Yesterday');
	});

	it('says older messages are loading, and that long work carries on without you', () => {
		expect(render(LoadingOlder).querySelector('[role="status"]')!.textContent).toContain('Loading older messages…');
		const note = render(BackgroundNote, { eta: '~4 min' }).querySelector('[role="status"]')!;
		expect(note.textContent).toContain('~4 min');
	});

	it('Boundary gives the reason and what each crew member can do instead', () => {
		const onclick = vi.fn();
		const root = render(Boundary, {
			title: 'I can’t move money',
			reason: 'Nomi never touches your bank account.',
			alternatives: [
				{ agent: 'dana', title: 'Track it as spending', onclick },
				{ agent: 'kala', title: 'Remind you to pay', href: '/reminders' },
			],
		});
		expect(root.textContent).toContain('Nomi never touches your bank account.');
		click(button(root, /Track it as spending/));
		expect(onclick).toHaveBeenCalledOnce();
		expect(root.querySelector('a')!.getAttribute('href')).toBe('/reminders');
	});
});

describe('build', () => {
	it('FirstRun asks for a name and continues only with one', () => {
		const oncontinue = vi.fn();
		const root = render(FirstRun, { name: 'Mario Prasetyo', oncontinue });
		type(named<HTMLInputElement>(root, 'input', /.*/), 'Mario');
		click(button(root, 'Continue'));
		expect(oncontinue).toHaveBeenCalledWith('Mario');
		expect(button(render(FirstRun, { oncontinue }), 'Continue').disabled).toBe(true);
	});

	it('ResizeHandle moves the split with the keyboard and resets on double-click', () => {
		const onresize = vi.fn();
		const handle = named(render(ResizeHandle, { width: 520, available: 1600, onresize }), '[role="separator"]', 'Resize chat');
		expect(handle.getAttribute('aria-valuenow')).toBe('520');
		key(handle, 'ArrowRight');
		expect(onresize).toHaveBeenLastCalledWith(536);
		handle.dispatchEvent(new MouseEvent('dblclick', { bubbles: true }));
		expect(onresize).toHaveBeenLastCalledWith(520);
	});

	it('PublishButton is quiet when up to date, publishes what is waiting and retries a failure', () => {
		expect(button(render(PublishButton, { state: 'idle' }), 'Up to date').disabled).toBe(true);
		const onpublish = vi.fn();
		click(button(render(PublishButton, { state: 'ready', count: 2, onpublish }), /Publish/));
		expect(onpublish).toHaveBeenCalledOnce();
		expect(button(render(PublishButton, { state: 'publishing', elapsed: '0:12' }), /Publishing/).getAttribute('aria-busy')).toBe('true');
		const onretry = vi.fn();
		click(button(render(PublishButton, { state: 'failed', onretry }), /Retry/));
		expect(onretry).toHaveBeenCalledOnce();
	});

	it('queues asks, lets the next one go first, and asks the publisher to publish', () => {
		const mario = { name: 'Mario Prasetyo', initials: 'MP' };
		const sari = { name: 'Sari Wulandari', initials: 'SW', peer: 1 };
		const onfirst = vi.fn();
		const queue = render(RequestQueue, {
			asks: [
				{ text: 'Make the Pre-order button green', who: mario, state: 'running', meta: 'Mario · running · 00:21' },
				{ text: 'Add a Makanan section', who: sari, state: 'next', meta: 'Sari · next' },
			],
			onfirst,
		});
		expect(queue.textContent).toContain('2 asks in line');
		click(button(queue, "Do Sari's first"));
		expect(onfirst).toHaveBeenCalledWith(1);
		const onpublish = vi.fn();
		const request = render(PublishRequest, { who: sari, meta: '2 files', onpublish, onreview: () => {} });
		expect(request.textContent).toContain("Sari's change is ready to publish");
		click(button(request, 'Publish'));
		expect(onpublish).toHaveBeenCalledOnce();
	});

	it('RoleMenu picks what an invited person can do, and BuildInvite links to the invite', () => {
		const onchange = vi.fn();
		const menu = render(RoleMenu, { value: 'build', onchange });
		click(named(menu, 'label', /Can prompt/).querySelector('input')!);
		expect(onchange).toHaveBeenCalledWith('prompt');
		const invite = render(BuildInvite, {
			from: 'Mario Prasetyo',
			project: 'warung-kopi-menu',
			detail: 'Menu and pre-order page',
			role: 'prompt',
			href: 'https://nomi.app/invite/abc',
			sentTo: 'sari.w@gmail.com',
			expires: 'in 7 days',
		});
		expect(invite.querySelector('h2')!.textContent).toBe('Mario Prasetyo invited you to build warung-kopi-menu');
		expect(named(invite, 'a', 'Join and build').getAttribute('href')).toBe('https://nomi.app/invite/abc');
	});

	it('says who joined and whom you follow, and stops following', () => {
		expect(render(JoinedNote, { name: 'Sari', role: 'prompt', publisher: 'Mario' }).querySelector('[role="separator"]')!.getAttribute('aria-label')).toBe(
			'Sari joined · can prompt · Mario publishes',
		);
		const onstop = vi.fn();
		const bar = render(FollowBar, { name: 'Sari', peer: 1, where: 'Code · +page.svelte', onstop });
		expect(bar.querySelector('[role="status"]')!.textContent!.replace(/\s+/g, ' ')).toContain('Following Sari · Code · +page.svelte');
		click(button(bar, 'Stop following'));
		expect(onstop).toHaveBeenCalledOnce();
	});
});

describe('mobile', () => {
	it('NavBar goes back', () => {
		const onback = vi.fn();
		const root = render(NavBar, { title: 'Food spending in October', who: { agent: 'dana', name: 'Dana' }, onback, onmenu: () => {} });
		expect(root.querySelector('h1, h2')!.textContent).toContain('Food spending in October');
		click(button(root, 'Back'));
		expect(onback).toHaveBeenCalledOnce();
	});

	it('the phone composer sends only with text, and stops a reply', () => {
		const onsend = vi.fn();
		const root = render(MobileComposer, { onsend, onattach: () => {}, onrecord: () => {} });
		expect(root.querySelectorAll('button[aria-label="Send"]')).toHaveLength(0);
		type(named<HTMLTextAreaElement>(root, 'textarea', 'Message'), 'Berapa pengeluaran makan?');
		click(button(root, 'Send'));
		expect(onsend).toHaveBeenCalledWith('Berapa pengeluaran makan?');
		const onstop = vi.fn();
		click(button(render(MobileComposer, { onsend, onstop, streaming: true }), 'Stop reply'));
		expect(onstop).toHaveBeenCalledOnce();
	});

	it('UserMessage says when it was not sent, with a retry', () => {
		const onretry = vi.fn();
		click(button(render(UserMessage, { text: 'Hi', status: 'failed', onretry }), 'Retry'));
		expect(onretry).toHaveBeenCalledOnce();
	});

	it('WorkspaceSwitch moves between Chat and Preview, and AskBar asks Koda', () => {
		const onchange = vi.fn();
		const toggle = render(WorkspaceSwitch, { value: 'chat', previewCount: 3, onchange });
		expect(named(toggle, '[role="radio"]', /Chat/).getAttribute('aria-checked')).toBe('true');
		click(named(toggle, '[role="radio"]', /Preview/));
		expect(onchange).toHaveBeenCalledWith('preview');
		const onask = vi.fn();
		click(button(render(AskBar, { onask, onvoice: () => {} }), 'Ask Koda about this screen'));
		expect(onask).toHaveBeenCalledOnce();
	});
});
