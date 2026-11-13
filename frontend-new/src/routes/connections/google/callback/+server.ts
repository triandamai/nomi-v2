import { redirect } from '@sveltejs/kit';
import { finishGoogle } from '$lib/server/connections';
import type { RequestHandler } from './$types';

/** Google sends the person back here after they allowed (or declined) access: finish, then go on. */
export const GET: RequestHandler = async (event) => {
	const [code, state] = [event.url.searchParams.get('code'), event.url.searchParams.get('state')];
	if (event.url.searchParams.get('error') || !code || !state) redirect(303, '/settings/connections?error=declined');
	const done = await finishGoogle(event, code, state);
	if (!done.ok) redirect(303, '/settings/connections?error=failed');
	redirect(303, done.data.resume_session_id ? `/chat/${done.data.resume_session_id}` : '/settings/connections?connected=google');
};
