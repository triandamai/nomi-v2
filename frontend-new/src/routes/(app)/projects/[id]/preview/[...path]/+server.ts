import { apiUrl } from '$lib/server/api';
import { openPreview } from '$lib/server/previewGrant';
import type { RequestHandler } from './$types';

// The frame opens `preview/~<grant>/`: redirecting it to drop the slash would send the page's
// relative links (`style.css`, `app.js`) one folder up.
export const trailingSlash = 'ignore';

/**
 * The project's own files, as the workspace's preview frame loads them (index.html, then what it
 * asks for, relative to it). They're code Koda wrote, so each document runs sandboxed, in an
 * origin of its own: its scripts run, but never as this app (its cookies, pages and actions),
 * however the file is opened. A sandboxed page sends no sign-in cookie, so the address carries a
 * grant for this project (`previewGrant.ts`); without one, or with someone else's, nothing is read.
 */
export const GET: RequestHandler = async (event) => {
	const [first = '', ...rest] = event.params.path.split('/');
	const token = first.startsWith('~') && event.params.path.includes('/') ? openPreview(first.slice(1), event.params.id) : null;
	// Only the project's own files: a `..` would leave the preview's folder on the backend.
	if (!token || rest.some((segment) => segment === '.' || segment === '..')) return new Response(null, { status: 403 });
	const id = encodeURIComponent(event.params.id);
	const path = rest.filter(Boolean).map(encodeURIComponent).join('/');
	const response = await event.fetch(apiUrl(path ? `/api/projects/${id}/preview/${path}` : `/api/projects/${id}/preview`), { headers: { Authorization: `Bearer ${token}` } }).catch(() => new Response(null, { status: 503 }));
	const headers = new Headers({
		'content-type': response.headers.get('content-type') ?? 'application/octet-stream',
		'cache-control': 'no-store',
		'content-security-policy': 'sandbox allow-scripts',
		'x-content-type-options': 'nosniff',
		// The grant is in the address: it isn't sent on to anything the page loads or links to.
		'referrer-policy': 'no-referrer',
	});
	return new Response(response.body, { status: response.status, headers });
};
