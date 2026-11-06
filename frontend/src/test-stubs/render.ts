// Test helpers for mounting a component or route page into jsdom and poking at it the way a person
// would: by the text and labels they see.
import { flushSync, mount, unmount, type Component } from 'svelte';
import { afterEach } from 'vitest';

const mounted: (() => void)[] = [];

afterEach(() => {
	while (mounted.length) mounted.pop()!();
	document.body.innerHTML = '';
});

export function render<P extends Record<string, unknown>>(component: Component<P>, props: P) {
	const target = document.body.appendChild(document.createElement('div'));
	const instance = mount(component, { target, props });
	flushSync();
	mounted.push(() => unmount(instance));
	return target;
}

/** Buttons, links and other controls whose visible text or aria-label contains `name`. */
export function controls(name: string | RegExp, root: ParentNode = document.body): HTMLElement[] {
	const matches = (value: string | null | undefined) =>
		!!value && (typeof name === 'string' ? value.includes(name) : name.test(value));
	return [...root.querySelectorAll<HTMLElement>('button, a, [role="button"], [role="tab"], [role="menuitem"], [role="option"], [role="switch"], input[type="checkbox"]')].filter(
		(el) => matches(el.getAttribute('aria-label')) || matches(el.textContent?.trim()) || matches(el.getAttribute('title')),
	);
}

/** The single control named `name`; throws when there's none so a test fails at the right line. */
export function control(name: string | RegExp, root: ParentNode = document.body): HTMLElement {
	const found = controls(name, root);
	if (!found.length) throw new Error(`no control named ${String(name)}`);
	return found[0];
}

export function click(el: Element | null | undefined) {
	if (!el) throw new Error('nothing to click');
	(el as HTMLElement).click();
	flushSync();
}

export function type(el: Element | null | undefined, value: string) {
	if (!el) throw new Error('nothing to type into');
	const input = el as HTMLInputElement;
	input.value = value;
	input.dispatchEvent(new Event('input', { bubbles: true }));
	input.dispatchEvent(new Event('change', { bubbles: true }));
	flushSync();
}

export function text(root: ParentNode = document.body): string {
	return (root.textContent ?? '').replace(/\s+/g, ' ');
}

/** A JSON `Response` like the backend's. */
export function json(body: unknown, status = 200) {
	return new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });
}
