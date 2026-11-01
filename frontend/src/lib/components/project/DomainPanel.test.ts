// @vitest-environment jsdom
import { afterEach, beforeAll, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, tick, unmount } from 'svelte';

const answers = vi.hoisted(() => ({ domain: null as unknown }));
vi.mock('$lib/projectApi', () => ({
	ProjectApiError: class extends Error {},
	projectApi: vi.fn(async (_id: string, path: string) => (path === 'domain' ? answers.domain : { available: true, suggestions: [] })),
}));

import DomainPanel from './DomainPanel.svelte';
import type { ProjectDeploy } from '$lib/projectDeploy.svelte';

// jsdom has no modal dialogs or ResizeObserver; the sheet only needs them to exist.
beforeAll(() => {
	globalThis.ResizeObserver ??= class {
		observe() {}
		unobserve() {}
		disconnect() {}
	};
	HTMLDialogElement.prototype.showModal ??= function (this: HTMLDialogElement) {
		this.open = true;
	};
	HTMLDialogElement.prototype.close ??= function (this: HTMLDialogElement) {
		this.open = false;
	};
});

let component: ReturnType<typeof mount> | null = null;

async function render(domain: unknown) {
	answers.domain = domain;
	const deploy = { projectId: 'p1', changes: 0, refresh: vi.fn() } as unknown as ProjectDeploy;
	component = mount(DomainPanel, { target: document.body, props: { deploy } });
	flushSync();
	// The address loads after mount.
	for (let i = 0; i < 3; i++) await tick();
	flushSync();
}

afterEach(() => {
	if (component) unmount(component);
	component = null;
	document.body.innerHTML = '';
});

const onOldDomain = {
	hostname: 'todo.apps.old.dev',
	subdomain: 'todo',
	url: 'https://todo.apps.old.dev',
	can_edit: true,
	base_domain: 'nomi.app',
	pending_hostname: 'todo.nomi.app',
};

describe('DomainPanel', () => {
	it('says where an address on an older domain moves, under the configured base domain', async () => {
		await render(onOldDomain);
		expect(document.body.textContent).toContain('todo.nomi.app');
		expect(document.body.textContent).toContain('Move now');
	});

	it('offers the configured base domain after the name when changing the address', async () => {
		await render(onOldDomain);
		const move = [...document.querySelectorAll('button')].find((b) => b.textContent?.includes('Move now'));
		move?.click();
		flushSync();
		expect(document.querySelector('.name__suffix')?.textContent).toBe('.nomi.app');
	});

	it('says nothing about moving once the address is under the base domain', async () => {
		await render({ ...onOldDomain, hostname: 'todo.nomi.app', url: 'https://todo.nomi.app', pending_hostname: null });
		expect(document.body.textContent).not.toContain('Move now');
	});
});
