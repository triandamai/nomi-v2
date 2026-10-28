// Helpers for component tests in the browser.

/** A token's colour as the browser computes it where `within` sits (so in that element's theme). */
export function tokenColor(token: string, within: Element = document.body): string {
	const probe = document.createElement('span');
	probe.style.color = `var(--${token})`;
	within.append(probe);
	const color = getComputedStyle(probe).color;
	probe.remove();
	return color;
}

/** A token's colour as the browser computes it for a background (the transparent ones stay rgba). */
export function tokenBackground(token: string, within: Element = document.body): string {
	const probe = document.createElement('span');
	probe.style.backgroundColor = `var(--${token})`;
	within.append(probe);
	const color = getComputedStyle(probe).backgroundColor;
	probe.remove();
	return color;
}
