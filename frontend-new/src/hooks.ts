import type { Reroute } from '@sveltejs/kit';
import { building } from '$app/environment';
import { env } from '$env/dynamic/public';
import { routeOf } from '$lib/docs/static-path';
import { domainsFrom, landingRoute } from '$lib/domains';

// In the static design-system copy, a page opened as components/button.html hydrates as the
// components/[slug] route with slug "button", not "button.html". Otherwise the landing domain's
// root is the landing page (/welcome), with its address left as it is.
export const reroute: Reroute = ({ url }) => {
	if (__DS_STATIC__) return routeOf(url.pathname);
	if (building) return undefined;
	return landingRoute(url, domainsFrom(env));
};
