/**
 * The backend writes links to the old app's addresses into emails and notifications (APP_URL +
 * path). v2 owns APP_URL, so each of those goes on to its v2 page. Paths v2 serves under the same
 * name (/register, /reset-password, /invite/:code, the OAuth callbacks) aren't here.
 */
const LEGACY: [pattern: RegExp, to: (match: RegExpMatchArray) => string][] = [
	[/^\/login$/, () => '/sign-in'],
	[/^\/organizations\/([^/]+)$/, ([, id]) => `/settings/workspaces/${id}`],
	[/^\/projects\/([^/]+)\/app$/, ([, id]) => `/projects/${id}`],
	[/^\/billing$/, () => '/settings/plan'],
	[/^\/models$/, () => '/settings/models'],
];

/**
 * Where an old-app address goes in v2, with its query kept; null for any other path. The query is
 * read only for a match: prerendered pages (the static docs) may not read it at all.
 */
export function legacyPath(url: { pathname: string; search: string }): string | null {
	for (const [pattern, to] of LEGACY) {
		const match = url.pathname.match(pattern);
		if (match) return `${to(match)}${url.search}`;
	}
	return null;
}
