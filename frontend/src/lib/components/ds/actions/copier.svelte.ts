// Copies text to the clipboard and remembers for a moment that it did, so the control can say
// "Copied". A refused clipboard (no permission, insecure page) simply doesn't claim it copied.

export function createCopier(ms = 2000) {
	let copied = $state(false);
	let timer: ReturnType<typeof setTimeout> | undefined;
	return {
		get copied() {
			return copied;
		},
		async copy(text: string) {
			try {
				await navigator.clipboard.writeText(text);
			} catch {
				return;
			}
			copied = true;
			clearTimeout(timer);
			timer = setTimeout(() => (copied = false), ms);
		},
	};
}
