import { json } from '@sveltejs/kit';
import { checkUsername } from '$lib/server/profile';
import { usernameProblem } from '$lib/settings/profile';
import type { RequestHandler } from './$types';

/** `?name=`: whether a username is free, asked as the person types it. */
export const GET: RequestHandler = async (event) => {
	const name = (event.url.searchParams.get('name') ?? '').trim().replace(/^@/, '');
	const problem = usernameProblem(name);
	if (!name || problem) return json({ available: false, reason: problem });
	return json(await checkUsername(event, name));
};
