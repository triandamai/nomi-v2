// Keeps a floating layer (the toasts) above everything else on the page, dialogs and sheets
// included. Those open in the browser's top layer, which no z-index reaches, and a modal one makes
// the rest of the page inert, so the layer is a manual popover moved into the topmost open modal
// (or the body when there is none) and shown again on top whenever a modal opens or closes.
// The element is moved out of where it was rendered: give it a wrapper of its own.

/** The modal dialog on top, or null. */
export function topModal(doc: Document = document): HTMLDialogElement | null {
	const modals = [...doc.querySelectorAll<HTMLDialogElement>('dialog[open]')].filter((dialog) => dialog.matches(':modal'));
	return modals.at(-1) ?? null;
}

/** Puts the layer on top of whatever is open now. */
export function raise(node: HTMLElement) {
	const host = topModal(node.ownerDocument) ?? node.ownerDocument.body;
	if (node.parentElement !== host) host.append(node);
	if (node.matches(':popover-open')) node.hidePopover();
	node.showPopover();
}

/** Attachment: keeps `node` (a `popover="manual"` element) above any dialog or sheet. */
export function topLayer(node: HTMLElement) {
	raise(node);
	let host = node.parentElement;
	// A modal opening or closing flips an `open`; one taken off the page takes the layer with it.
	const observer = new MutationObserver(() => {
		const next = topModal(node.ownerDocument) ?? node.ownerDocument.body;
		if (next === host && node.isConnected) return;
		raise(node);
		host = node.parentElement;
	});
	observer.observe(node.ownerDocument.body, { subtree: true, childList: true, attributeFilter: ['open'] });
	return () => {
		observer.disconnect();
		node.remove();
	};
}
