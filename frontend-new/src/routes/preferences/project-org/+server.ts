import { json } from '@sveltejs/kit';
import { chooseProjectsWorkspace } from '$lib/server/account';
import type { RequestHandler } from './$types';

/** The workspace switcher: `PUT { org_id }` picks the workspace projects are listed from. */
export const PUT: RequestHandler = async (event) => {
	if (!event.locals.accessToken) return new Response(null, { status: 401 });
	const body = (await event.request.json().catch(() => ({}))) as { org_id?: unknown };
	if (typeof body.org_id !== 'string' || !body.org_id) return new Response(null, { status: 400 });
	const result = await chooseProjectsWorkspace(event, body.org_id);
	return result.ok ? new Response(null, { status: 204 }) : json({ message: result.message ?? null }, { status: result.status });
};
