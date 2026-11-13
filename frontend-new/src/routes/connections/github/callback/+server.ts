import { redirect } from '@sveltejs/kit';
import { GITHUB_BACK_COOKIE as BACK, finishGithub } from '$lib/server/connections';
import type { RequestHandler } from './$types';

/** GitHub sends the person back here after they allowed Nomi: finish, then back to where they were. */
export const GET: RequestHandler = async (event) => {
	const [code, state] = [event.url.searchParams.get('code'), event.url.searchParams.get('state')];
	const back = event.cookies.get(BACK);
	event.cookies.delete(BACK, { path: '/' });
	if (event.url.searchParams.get('error') || !code || !state) redirect(303, '/settings/connections?error=declined');
	const done = await finishGithub(event, code, state);
	if (!done.ok) redirect(303, '/settings/connections?error=failed');
	redirect(303, back && back.startsWith('/') && !back.startsWith('//') ? back : '/settings/connections?connected=github');
};
