/**
 * Whether a link to `href` is the page that's loading now (`to` is where the navigation is going,
 * from `navigating.to` in `$app/state`). Path and search have to match; the hash doesn't matter.
 */
export function opensPage(href: string, to: URL | null | undefined): boolean {
	if (!to) return false;
	const target = new URL(href, to);
	return target.origin === to.origin && target.pathname === to.pathname && target.search === to.search;
}
