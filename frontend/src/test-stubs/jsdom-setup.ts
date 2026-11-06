// Browser APIs jsdom leaves out that Nomi's components reach for. Each is a quiet stand-in: enough
// for a component to mount and react, nothing more. Only filled in where jsdom has no version.
if (typeof window !== 'undefined') {
	window.matchMedia ??= ((query: string) => ({
		matches: false,
		media: query,
		onchange: null,
		addListener() {},
		removeListener() {},
		addEventListener() {},
		removeEventListener() {},
		dispatchEvent: () => false,
	})) as unknown as typeof window.matchMedia;

	class QuietObserver {
		observe() {}
		unobserve() {}
		disconnect() {}
		takeRecords() {
			return [];
		}
	}
	(globalThis as Record<string, unknown>).ResizeObserver ??= QuietObserver;
	(globalThis as Record<string, unknown>).IntersectionObserver ??= QuietObserver;

	Element.prototype.scrollIntoView ??= function () {};
	Element.prototype.scrollTo ??= function () {} as typeof Element.prototype.scrollTo;
	window.scrollTo = (() => {}) as typeof window.scrollTo;

	const dialog = globalThis.HTMLDialogElement?.prototype;
	if (dialog) {
		dialog.showModal ??= function (this: HTMLDialogElement) {
			this.setAttribute('open', '');
		};
		dialog.show ??= function (this: HTMLDialogElement) {
			this.setAttribute('open', '');
		};
		dialog.close ??= function (this: HTMLDialogElement) {
			this.removeAttribute('open');
			this.dispatchEvent(new Event('close'));
		};
	}
	(HTMLElement.prototype as unknown as Record<string, unknown>).showPopover ??= function () {};
	(HTMLElement.prototype as unknown as Record<string, unknown>).hidePopover ??= function () {};
}

// No network in component tests: a relative `fetch` (a form action, `/api/...`) answers like a
// SvelteKit action that succeeded with nothing to say. Tests that care install their own fake.
if (typeof window !== 'undefined') {
	globalThis.fetch = (async () =>
		new Response(JSON.stringify({ type: 'success', status: 200, data: '[]' }), {
			status: 200,
			headers: { 'content-type': 'application/json' },
		})) as typeof fetch;
}

// Copy buttons: jsdom has no clipboard.
if (typeof navigator !== 'undefined' && !navigator.clipboard) {
	Object.defineProperty(navigator, 'clipboard', { value: { writeText: async () => {}, readText: async () => '' }, configurable: true });
}

// Sockets never connect in component tests (and jsdom refuses relative socket URLs): a quiet
// stand-in. Tests that drive a socket install their own.
if (typeof window !== 'undefined') {
	class QuietSocket extends EventTarget {
		static CONNECTING = 0;
		static OPEN = 1;
		static CLOSING = 2;
		static CLOSED = 3;
		readyState = 0;
		url: string;
		constructor(url: string | URL) {
			super();
			this.url = String(url);
		}
		send() {}
		close() {
			this.readyState = 3;
		}
	}
	(globalThis as Record<string, unknown>).WebSocket = QuietSocket;
}

// Downloads and previews make object URLs; jsdom's Blob doesn't survive Node's own
// URL.createObjectURL, so they get a placeholder address.
if (typeof window !== 'undefined') {
	URL.createObjectURL = () => 'blob:test';
	URL.revokeObjectURL = () => {};
}
