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
