import { json } from '@sveltejs/kit';
import { createWorkspace } from '$lib/server/account';
import type { RequestHandler } from './$types';

/** The workspace switcher's "Create workspace": `POST { name }` → 201 with the new workspace, or
 * the status with which field was wrong (`required`, `long`) or the backend's own reason. */
export const POST: RequestHandler = async (event) => {
	if (!event.locals.accessToken) return new Response(null, { status: 401 });
	const body = (await event.request.json().catch(() => ({}))) as { name?: unknown };
	const result = await createWorkspace(event, typeof body.name === 'string' ? body.name : '');
	if (result.ok) return json(result.org, { status: 201 });
	return json({ field: result.field ?? null, message: result.message ?? null }, { status: result.status });
};
