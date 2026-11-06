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
	const found = [...root.querySelectorAll<HTMLElement>('button, a, [role="button"], [role="tab"], [role="menuitem"], [role="option"], [role="switch"], input[type="checkbox"]')].filter(
		(el) => matches(el.getAttribute('aria-label')) || matches(el.textContent?.trim()) || matches(el.getAttribute('title')),
	);
	// Exactly-named controls first: "Delete" before "Delete endpoint https://…".
	const exact = (el: HTMLElement) => typeof name === 'string' && (el.textContent?.trim() === name || el.getAttribute('aria-label') === name);
	return [...found.filter(exact), ...found.filter((el) => !exact(el))];
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

export type Route = [method: string, path: string | RegExp, answer: unknown | ((body: unknown, url: string) => unknown)];

/**
 * Replaces `fetch` with a small fake backend: the first route whose method and path match answers
 * (a function gets the parsed body). Unmatched calls answer 404. Returns the calls made, in order.
 */
export function fakeBackend(routes: Route[], { strip = '' }: { strip?: string } = {}) {
	const calls: { method: string; path: string; body: unknown }[] = [];
	const fetchMock = async (input: RequestInfo | URL, init?: RequestInit) => {
		const url = String(input instanceof Request ? input.url : input);
		const method = (init?.method ?? (input instanceof Request ? input.method : 'GET')).toUpperCase();
		const path = url.replace(/^https?:\/\/[^/]+/, '').replace(strip, '');
		let body: unknown = undefined;
		if (typeof init?.body === 'string') {
			try {
				body = JSON.parse(init.body);
			} catch {
				body = init.body;
			}
		} else if (init?.body instanceof FormData) {
			body = Object.fromEntries(init.body.entries());
		}
		calls.push({ method, path, body });
		for (const [m, p, answer] of routes) {
			if (m !== '*' && m !== method) continue;
			if (typeof p === 'string' ? p !== path.split('?')[0] && p !== path : !p.test(path)) continue;
			const value = typeof answer === 'function' ? (answer as (b: unknown, u: string) => unknown)(body, path) : answer;
			if (value instanceof Response) return value;
			return json(value ?? null);
		}
		return new Response('not found', { status: 404 });
	};
	globalThis.fetch = fetchMock as typeof fetch;
	return calls;
}

/** Lets pending fetches and effects settle. */
export async function settle(times = 5) {
	for (let i = 0; i < times; i++) {
		await new Promise((resolve) => setTimeout(resolve, 0));
		flushSync();
	}
}

/**
 * Clicks every button under `root` once (not submit buttons or links), the way someone trying
 * the screen would, keeping going past any that fail. Returns how many were clicked.
 */
export function clickEverything(root: ParentNode = document.body, skip: RegExp = /delete|remove|sign out|log out|forget/i): number {
	let clicked = 0;
	const seen = new Set<Element>();
	for (let round = 0; round < 2; round++) {
		for (const el of [...root.querySelectorAll<HTMLElement>('button:not([type="submit"]), [role="button"], [role="tab"], [role="switch"], summary')]) {
			if (seen.has(el) || !el.isConnected || (el as HTMLButtonElement).disabled) continue;
			if (skip.test(el.textContent ?? '') || skip.test(el.getAttribute('aria-label') ?? '')) continue;
			seen.add(el);
			try {
				el.click();
				flushSync();
				clicked++;
			} catch {
				// a control that needs more than a click; carry on
			}
		}
	}
	return clicked;
}
