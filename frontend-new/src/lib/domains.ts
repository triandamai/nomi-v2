/**
 * The landing page and the app are one server on two domains: the landing domain (nomi.build)
 * shows /welcome at its root and nothing else; the app domain (app.nomi.build) is everything else.
 * With either host unset (dev, previews) there is one domain and every page is on it.
 */
export interface Domains {
	landing?: string;
	app?: string;
}

/** The route the landing page lives at in this app. */
export const LANDING_ROUTE = '/welcome';

/** Built app files and the health check, which the landing page's own host must keep serving. */
const SHARED_PREFIXES = ['/_app/', '/health'];

/** A file from static/ (favicon, fonts, images) or a page's data (__data.json): it has an extension. */
const FILE = /\.[a-z0-9]+$/i;

const split = (domains: Domains): domains is Required<Domains> => Boolean(domains.landing && domains.app);

/** The route to render for this address: the landing page at the landing domain's root. */
export function landingRoute(url: { hostname: string; pathname: string }, domains: Domains): string | undefined {
	if (split(domains) && url.hostname === domains.landing && url.pathname === '/') return LANDING_ROUTE;
	return undefined;
}

/** The two hosts from the environment (PUBLIC_LANDING_HOST, PUBLIC_APP_HOST). */
export const domainsFrom = (env: Record<string, string | undefined>): Domains => ({
	landing: env.PUBLIC_LANDING_HOST || undefined,
	app: env.PUBLIC_APP_HOST || undefined,
});

/** An app page's address as the landing page links to it: on the app domain when the two are split. */
export function appHref(path: string, url: { hostname: string; protocol: string }, domains: Domains): string {
	return onLanding(url.hostname, domains) ? `${url.protocol}//${domains.app}${path}` : path;
}

/** The landing page's own address: the landing domain's root when split, else /welcome. */
export const landingHome = (hostname: string, domains: Domains) => (onLanding(hostname, domains) ? '/' : LANDING_ROUTE);

/** Whether this request is for the landing domain. */
export const onLanding = (hostname: string, domains: Domains) => split(domains) && hostname === domains.landing;

/**
 * Where to send a request that's on the wrong domain, or null to serve it here: the landing domain
 * sends app pages to the app domain, and the app domain sends /welcome to the landing domain's root.
 */
export function crossDomainRedirect(url: { hostname: string; pathname: string; search: string; protocol: string }, domains: Domains): string | null {
	if (!split(domains)) return null;
	const { hostname, pathname, protocol } = url;
	if (hostname === domains.landing) {
		if (pathname === '/' || SHARED_PREFIXES.some((prefix) => pathname.startsWith(prefix)) || FILE.test(pathname)) return null;
		if (pathname === LANDING_ROUTE) return '/';
		return `${protocol}//${domains.app}${pathname}${url.search}`;
	}
	if (hostname === domains.app && pathname === LANDING_ROUTE) return `${protocol}//${domains.landing}/`;
	return null;
}
