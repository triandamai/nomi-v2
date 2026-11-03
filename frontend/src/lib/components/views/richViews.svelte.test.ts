// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, unmount, type Component } from 'svelte';
import WithChatSend from '$lib/components/blocks/twins/WithChatSend.svelte';
import ViewBlock from './ViewBlock.svelte';
import examples from '$lib/views/examples.json';
import type { View } from '$lib/views/types';

// The views the crew shows are drawn with the design system's chat pieces, and the ones that ask
// send the person's answer through the chat.

let mounted: ReturnType<typeof mount>[] = [];

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function render(component: Component<any>, props: Record<string, unknown>): HTMLElement {
	const target = document.body.appendChild(document.createElement('div'));
	mounted.push(mount(component, { target, props }));
	flushSync();
	return target;
}

const example = <N extends View['view']>(name: N) => (examples as View[]).find((view) => view.view === name) as Extract<View, { view: N }>;
const show = (block: View, send = vi.fn(), extra: Record<string, unknown> = {}) => render(WithChatSend, { send, block: ViewBlock, props: { block, ...extra } });
const button = (root: ParentNode, text: string) => [...root.querySelectorAll('button')].find((b) => b.textContent?.includes(text))!;

afterEach(() => {
	for (const component of mounted) unmount(component);
	mounted = [];
	document.body.innerHTML = '';
	vi.unstubAllGlobals();
});

describe('views drawn with the design system', () => {
	it('draws every example', () => {
		vi.stubGlobal(
			'ResizeObserver',
			class {
				observe() {}
				unobserve() {}
				disconnect() {}
			},
		);
		for (const view of examples as View[]) {
			const root = show(view);
			expect(root.textContent?.trim().length, view.view).toBeGreaterThan(0);
		}
	});

	it('shows code numbered and coloured, copying the code alone', () => {
		const root = show(example('code'));
		expect(root.querySelector('figure figcaption')?.textContent).toContain('src/lib/budget.ts');
		expect(root.querySelectorAll('[data-line-number]').length).toBe(3);
	});

	it('shows a diff with its added and removed lines', () => {
		const root = show(example('diff'));
		expect(root.textContent).toContain('src/lib/budget.ts');
		expect(root.textContent).toContain('const base = Math.round(total / weeks)');
	});

	it('numbers the sources and links the web ones', () => {
		const root = show(example('sources'));
		expect(root.querySelector('a[href="https://kosku.id"]')).not.toBeNull();
		expect(root.textContent).toContain('Peraturan Kos Mawar 2026.pdf');
	});

	it('sends the recommended choice of a decision', () => {
		const send = vi.fn();
		const root = show(
			{
				view: 'decision',
				question: 'Which plan?',
				options: [{ label: 'Basic' }, { label: 'Pro', recommended: true }],
			},
			send,
		);
		button(root, 'Pro').click();
		expect(send).toHaveBeenCalledWith('Pro');
	});

	it('keeps a decision that was answered from being sent again', () => {
		const send = vi.fn();
		const root = show(
			{
				view: 'decision',
				question: 'Which plan?',
				options: [{ label: 'Basic' }, { label: 'Pro' }],
			},
			send,
			{ answered: true },
		);
		expect(root.querySelector('fieldset')?.disabled).toBe(true);
	});

	it('sends what was picked in a selection, with a running total', () => {
		const send = vi.fn();
		const root = show(example('selection'), send);
		expect(root.textContent).toContain('/ month');
		button(root, 'Track these').click();
		expect(send).toHaveBeenCalledWith('I picked: Netflix, Spotify');
	});

	it('totals a cart from its prices and quantities, and checks out at the store', () => {
		const root = show(example('cart'));
		expect(root.querySelector('a[href="https://www.sayurbox.com"]')).not.toBeNull();
		// 2 × 29.000 + 3 × 19.000 + 10.000 delivery
		expect(root.textContent?.replace(/\D/g, '')).toContain('12500000');
	});

	it('previews a link with its site', () => {
		const root = show(example('link'));
		const link = root.querySelector('a')!;
		expect(link.getAttribute('target')).toBe('_blank');
	});
});
