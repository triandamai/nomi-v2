import type { Reroute } from '@sveltejs/kit';
import { routeOf } from '$lib/docs/static-path';

// In the static design-system copy, a page opened as components/button.html hydrates as the
// components/[slug] route with slug "button", not "button.html".
export const reroute: Reroute = ({ url }) => (__DS_STATIC__ ? routeOf(url.pathname) : undefined);
