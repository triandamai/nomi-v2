import { apiUrl } from '$lib/server/api';
import type { RequestHandler } from './$types';

// A design preview's files, for the Design Studio's frame (see $lib/designStudio). The frame is
// sandboxed into an origin of its own, so the version's unguessable token is the whole
// permission: the person's sign-in is never sent on, and the answer carries the headers that
// keep the page fenced in wherever its address is opened.

// A page links to its neighbours relatively ("menu/"), so the address is served as it is asked.
export const trailingSlash = 'ignore';

const TOKEN = /^[0-9a-f]{64}$/;
/** Used when the answer has no policy of its own: no page of a preview is ever served unfenced. */
const FALLBACK_POLICY = "sandbox allow-scripts; connect-src 'none'; form-action 'none'; base-uri 'none'; object-src 'none'; frame-src 'none'";
/** The answer's headers that are passed on. */
const KEPT = ['content-type', 'referrer-policy', 'x-content-type-options', 'cache-control', 'access-control-allow-origin', 'cross-origin-resource-policy'];

export const GET: RequestHandler = async ({ params, fetch }) => {
	if (!TOKEN.test(params.token)) return new Response('not found', { status: 404 });
	const answer = await fetch(apiUrl(`/api/design/preview/${params.token}/${params.path ?? ''}`));
	const headers = new Headers();
	for (const name of KEPT) {
		const value = answer.headers.get(name);
		if (value) headers.set(name, value);
	}
	headers.set('content-security-policy', answer.headers.get('content-security-policy') ?? FALLBACK_POLICY);
	return new Response(answer.body, { status: answer.status, headers });
};
