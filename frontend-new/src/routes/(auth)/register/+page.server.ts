import { fail, redirect } from '@sveltejs/kit';
import { displayEmail } from '$lib/auth/format';
import { googleErrorFrom, type Alert, type Fields } from '$lib/auth/problems';
import { inviteCodeFrom, withRedirect } from '$lib/auth/redirects';
import { getLocale } from '$lib/paraglide/runtime';
import { alertFor, callAuth, emailProblem, googleAvailable, lastMethod, newPasswordProblem, rememberMethod } from '$lib/server/authFlow';
import { completeSignIn, safeRedirect, setPendingSignIn, type TokenPair, type Verification } from '$lib/server/signIn';
import type { Actions, PageServerLoad } from './$types';

// Create an account: email, password and a workspace, or the workspace of the invite the person
// came from (the backend's invite emails link here with `redirect_to=/invite/:code&email=`).

export const load: PageServerLoad = async ({ url, cookies, locals, fetch }) => {
	const target = safeRedirect(url.searchParams.get('redirect_to'));
	if (locals.accessToken) redirect(303, target);
	return {
		google: await googleAvailable(fetch),
		googleError: googleErrorFrom(url.searchParams.get('google_error')),
		lastUsed: lastMethod(cookies),
		email: url.searchParams.get('email') ?? '',
		invite: inviteCodeFrom(target),
		redirectTo: target === '/' ? null : target,
	};
};

type Values = { email: string; workspace: string };
const failure = (status: number, values: Values, fields: Fields, alert: Alert | null = null) => fail(status, { values, fields, alert });

export const actions: Actions = {
	default: async (event) => {
		const redirectTo = safeRedirect(event.url.searchParams.get('redirect_to'));
		const invite = inviteCodeFrom(redirectTo);
		const data = await event.request.formData();
		const email = String(data.get('email') ?? '').trim();
		const password = String(data.get('password') ?? '');
		const workspace = String(data.get('workspace') ?? '').trim();
		const values = { email, workspace };

		const fields: Fields = {};
		const emailWrong = emailProblem(email);
		if (emailWrong) fields.email = emailWrong;
		const passwordWrong = newPasswordProblem(password);
		if (passwordWrong) fields.password = passwordWrong;
		if (!invite && !workspace) fields.workspace = 'workspace_required';
		if (Object.keys(fields).length) return failure(400, values, fields);

		const org = invite ? { mode: 'join', invite_code: invite } : { mode: 'create', name: workspace };
		const response = await callAuth(event.fetch, '/api/auth/register', { email, password, org, language: getLocale() });
		if (!response) return failure(503, values, {}, { kind: 'unreachable' });
		if (response.status === 409) return failure(409, values, { email: 'email_taken' });
		if (response.status === 400 && invite) return failure(400, values, { invite: 'invite_invalid' });
		// The account exists; signing in sends a new code.
		if (response.status === 502) return failure(502, values, {}, { kind: 'register_email_failed', status: 502 });
		if (!response.ok) return failure(response.status, values, {}, await alertFor(response, 'register_failed'));

		const body = (await response.json()) as TokenPair | { verification: Verification };
		// A new account confirms its email with a code first; sign-up finishes on /verify.
		if ('verification' in body) {
			setPendingSignIn(event.cookies, { challengeId: body.verification.challenge_id, email: displayEmail(body.verification.email), isNew: true, redirectTo });
			redirect(303, '/verify');
		}
		await completeSignIn(event, body, true);
		rememberMethod(event.cookies, 'email');
		redirect(303, withRedirect('/first-run', redirectTo));
	},
};
