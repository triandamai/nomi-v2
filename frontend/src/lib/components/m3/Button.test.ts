// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { createRawSnippet, mount, unmount } from 'svelte';

const nav = vi.hoisted(() => ({ navigating: { to: null as { url: URL } | null } }));
vi.mock('$app/state', () => nav);

const { default: Button } = await import('./Button.svelte');

const label = createRawSnippet(() => ({ render: () => '<span>Sign in</span>' }));
let instance: Record<string, unknown> | undefined;
afterEach(() => {
	if (instance) unmount(instance);
	instance = undefined;
	nav.navigating.to = null;
	document.body.innerHTML = '';
});

function render(props: Record<string, unknown>) {
	const target = document.body.appendChild(document.createElement('div'));
	instance = mount(Button, { target, props: { children: label, ...props } });
	return target;
}

describe('Button loading', () => {
	it('spins and refuses a second press while loading', () => {
		const button = render({ type: 'submit', loading: true }).querySelector('button')!;
		expect(button.disabled).toBe(true);
		expect(button.getAttribute('aria-busy')).toBe('true');
		expect(button.querySelector('.m3-button__spinner')).not.toBeNull();
	});

	it('is a plain button otherwise', () => {
		const button = render({ type: 'submit' }).querySelector('button')!;
		expect(button.disabled).toBe(false);
		expect(button.hasAttribute('aria-busy')).toBe(false);
		expect(button.querySelector('.m3-button__spinner')).toBeNull();
	});

	it('spins on a link while the page it opens loads', () => {
		nav.navigating.to = { url: new URL('http://localhost/register') };
		const link = render({ href: '/register' }).querySelector('a')!;
		expect(link.getAttribute('aria-busy')).toBe('true');
		expect(link.querySelector('.m3-button__spinner')).not.toBeNull();
	});

	it('leaves other links alone', () => {
		nav.navigating.to = { url: new URL('http://localhost/login') };
		const link = render({ href: '/register' }).querySelector('a')!;
		expect(link.hasAttribute('aria-busy')).toBe(false);
	});
});
