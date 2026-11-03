import { error, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { PageServerLoad } from './$types';

/** GitHub sends the person back here after they allowed Nomi: finish connecting, then go back. */
export const load: PageServerLoad = async ({ url, cookies, fetch }) => {
	const code = url.searchParams.get('code');
	const state = url.searchParams.get('state');
	if (!code || !state) throw error(400, 'GitHub sent no code');
	const response = await apiFetch(fetch, cookies, '/api/connections/github/callback', { method: 'POST', body: JSON.stringify({ code, state }) });
	if (!response.ok) throw error(response.status, 'could not connect GitHub');
	const back = cookies.get('nomi_github_back');
	cookies.delete('nomi_github_back', { path: '/' });
	throw redirect(303, back && back.startsWith('/') && !back.startsWith('//') ? back : '/connections');
};
