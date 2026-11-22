import { readFileSync } from 'node:fs';
import { beforeEach, describe, expect, it, vi } from 'vitest';

// app.html's guard for pages left over from a previous deploy: when an app file fails to load it
// loads the page again, once a minute at most. The failure itself still reaches SvelteKit, which
// shows its error page (or loads the new build) rather than crashing on a missing module.

const html = readFileSync(new URL('./app.html', import.meta.url), 'utf8');
const script = [...html.matchAll(/<script>([\s\S]*?)<\/script>/g)].map((m) => m[1]).find((s) => s.includes('nomi-stale-build-reload'))!;

let listeners: Record<string, (event: Event) => void>;
let reload: ReturnType<typeof vi.fn>;
let storage: Map<string, string>;

function runGuard() {
	listeners = {};
	reload = vi.fn();
	const sessionStorage = { getItem: (k: string) => storage.get(k) ?? null, setItem: (k: string, v: string) => void storage.set(k, v) };
	new Function('addEventListener', 'location', 'sessionStorage', script)(
		(type: string, listener: (event: Event) => void) => (listeners[type] = listener),
		{ reload },
		sessionStorage,
	);
}

function preloadError(): Event {
	const event = new Event('vite:preloadError', { cancelable: true });
	listeners['vite:preloadError'](event);
	return event;
}

beforeEach(() => {
	storage = new Map();
	runGuard();
});

describe('stale build reload', () => {
	it('loads the page again when an app file fails to load', () => {
		preloadError();
		expect(reload).toHaveBeenCalledOnce();
	});

	it('lets the failure reach SvelteKit instead of handing it a missing module', () => {
		expect(preloadError().defaultPrevented).toBe(false);
	});

	it('does not loop: a second failure within a minute is left to SvelteKit', () => {
		preloadError();
		runGuard();
		const second = preloadError();
		expect(reload).not.toHaveBeenCalled();
		expect(second.defaultPrevented).toBe(false);
	});
});
