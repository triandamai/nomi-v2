// @vitest-environment jsdom
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, tick, unmount } from 'svelte';
import type { ModelChoice, ModelGroup, ModelOptions } from '$lib/modelPicker';

const postAction = vi.fn();
const reloadData = vi.fn(async () => {});
vi.mock('$lib/providerActions', () => ({ postAction: (...args: unknown[]) => postAction(...args), reloadData: () => reloadData() }));

const { default: DefaultModelSection } = await import('./DefaultModelSection.svelte');

beforeAll(() => {
	HTMLDialogElement.prototype.showModal = function (this: HTMLDialogElement) {
		this.setAttribute('open', '');
	};
	HTMLDialogElement.prototype.close = function (this: HTMLDialogElement) {
		this.removeAttribute('open');
		this.dispatchEvent(new Event('close'));
	};
});

const groups: ModelGroup[] = [
	{
		id: 'nomi:Anthropic',
		source: 'nomi',
		provider_id: 'anthropic',
		has_logo: true,
		label: 'Anthropic',
		connection_id: null,
		error: null,
		models: [
			{ key: 'admin:a1', label: 'Claude Sonnet', model_id: 'claude-sonnet', free: false, enabled: true },
			{ key: 'admin:a2', label: 'Claude Haiku', model_id: 'claude-haiku', free: false, enabled: true },
		],
	},
];

const sonnet: ModelChoice = {
	key: 'admin:a1',
	label: 'Claude Sonnet',
	model_id: 'claude-sonnet',
	provider_id: 'anthropic',
	provider_label: 'Anthropic',
	has_logo: true,
	chat_only: false,
};
const haiku: ModelChoice = { ...sonnet, key: 'admin:a2', label: 'Claude Haiku', model_id: 'claude-haiku' };

let component: ReturnType<typeof mount> | null = null;

function render(options: ModelOptions | null = { groups, current: sonnet }, onchange = vi.fn()) {
	const target = document.createElement('div');
	document.body.append(target);
	component = mount(DefaultModelSection, { target, props: { options, onchange } });
	flushSync();
	return target;
}

beforeEach(() => {
	postAction.mockReset();
	reloadData.mockClear();
});

afterEach(() => {
	if (component) unmount(component);
	component = null;
	document.body.innerHTML = '';
});

const trigger = (root: ParentNode) => root.querySelector<HTMLButtonElement>('button[aria-haspopup="listbox"]');
const alert = (root: ParentNode) => root.querySelector('section [role="alert"]')?.textContent?.trim() ?? null;

async function pick(root: ParentNode, name: string) {
	trigger(root)!.click();
	flushSync();
	await tick();
	flushSync();
	[...document.querySelectorAll<HTMLElement>('[role="option"]')].find((o) => o.textContent?.includes(name))!.click();
	flushSync();
}

async function settle() {
	for (let i = 0; i < 5; i++) await Promise.resolve();
	await tick();
	flushSync();
}

describe('DefaultModelSection', () => {
	it('explains the default and shows it on the picker', () => {
		const target = render();
		expect(target.textContent).toContain('New chats start on this model');
		expect(trigger(target)!.textContent).toContain('Claude Sonnet');
	});

	it('saves a pick as the default, showing it straight away', async () => {
		let resolve!: (value: unknown) => void;
		postAction.mockReturnValue(new Promise((r) => (resolve = r)));
		const onchange = vi.fn();
		const target = render(undefined, onchange);
		await pick(target, 'Claude Haiku');
		expect(trigger(target)!.textContent).toContain('Claude Haiku');
		const [name, body] = postAction.mock.calls[0] as [string, FormData];
		expect(name).toBe('setDefaultModel');
		expect(body.get('key')).toBe('admin:a2');
		resolve({ ok: true, data: { defaultModel: haiku } });
		await settle();
		expect(trigger(target)!.textContent).toContain('Claude Haiku');
		expect(onchange).toHaveBeenLastCalledWith(haiku);
		expect(alert(target)).toBeNull();
	});

	it("puts the old default back and says so when it can't be saved", async () => {
		postAction.mockResolvedValue({ ok: false, error: 'failed', models: [] });
		const onchange = vi.fn();
		const target = render(undefined, onchange);
		await pick(target, 'Claude Haiku');
		await settle();
		expect(trigger(target)!.textContent).toContain('Claude Sonnet');
		expect(alert(target)).toBe("Couldn't change your default model. Try again.");
		expect(onchange).toHaveBeenLastCalledWith(sonnet);
	});

	it('opens Manage models from its own button', async () => {
		const target = render();
		const manage = [...target.querySelectorAll('button')].find((b) => b.textContent?.trim() === 'Manage models')!;
		manage.click();
		flushSync();
		await tick();
		expect(document.querySelector('dialog[open]')?.textContent).toContain('Manage models');
	});

	it('works with only Nomi’s models and no default yet', () => {
		const target = render({ groups, current: null });
		expect(trigger(target)!.textContent).toContain('Choose model');
	});

	it("says the models couldn't be loaded instead of an empty picker", () => {
		const target = render(null);
		expect(trigger(target)).toBeNull();
		expect(target.textContent).toContain("Your models couldn't be loaded");
		expect([...target.querySelectorAll('button')].some((b) => b.textContent?.trim() === 'Manage models')).toBe(false);
	});
});
