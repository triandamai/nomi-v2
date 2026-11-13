import { error, fail, redirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import { connectedApps } from '$lib/settings/connections';
import { GOOGLE_SERVICES } from '$lib/settings/connections';
import * as connections from '$lib/server/connections';
import type { Sent } from '$lib/server/connections';
import type { Actions, PageServerLoad } from './$types';

/**
 * Settings › Connections (App / Settings / Connections OBmgS): Google Workspace, GitHub and Hevy.
 * `?resume=<chat>` comes from a chat's Connect prompt: connecting Google goes back to that chat.
 */
export const load: PageServerLoad = async (event) => {
	event.depends('app:connections');
	const data = await connections.loadConnections(event);
	if (!data.overview && !data.github) error(503, { message: m.connections_load_failed() });
	const count = connectedApps(data.overview, data.github).length;
	return { title: m.settings_connections(), meta: count === 1 ? m.connections_meta_one() : m.connections_meta({ count }), resume: event.url.searchParams.get('resume'), ...data };
};

const field = (form: FormData, name: string) => String(form.get(name) ?? '').trim();
/** The backend's reason: Hevy's are codes, worded here; GitHub's and Google's come as words. */
const SAID: Record<string, () => string> = { key_rejected: m.connections_key_rejected, unreachable: m.connections_unreachable, missing_key: m.connections_key_needed };
const refused = (sent: Extract<Sent<unknown>, { ok: false }>, app: string) =>
	fail(sent.status, { app, message: (sent.code && SAID[sent.code]?.()) || sent.message || m.action_failed() });
const isChat = (id: string) => /^[0-9a-f-]{36}$/.test(id);

export const actions: Actions = {
	/** Off to Google with the services chosen; it comes back through /connections/google/callback. */
	google: async (event) => {
		const form = await event.request.formData();
		const services = form.getAll('service').map(String).filter((service) => (GOOGLE_SERVICES as readonly string[]).includes(service));
		if (!services.length) return fail(400, { app: 'google', message: m.connections_pick_service() });
		const resume = field(form, 'resume');
		const started = await connections.startGoogle(event, services, isChat(resume) ? resume : null);
		if (!started.ok) return refused(started, 'google');
		redirect(303, started.data.url);
	},
	/** Off to GitHub; it comes back through /connections/github/callback. */
	github: async (event) => {
		const resume = field(await event.request.formData(), 'resume');
		const started = await connections.startGithub(event);
		if (!started.ok) return refused(started, 'github');
		if (isChat(resume)) event.cookies.set(connections.GITHUB_BACK_COOKIE, `/chat/${resume}`, { path: '/', httpOnly: true, sameSite: 'lax', maxAge: 600 });
		redirect(303, started.data.url);
	},
	githubToken: async (event) => {
		const token = field(await event.request.formData(), 'token');
		if (!token) return fail(400, { app: 'github', message: m.connections_token_needed() });
		const sent = await connections.githubToken(event, token);
		return sent.ok ? { connected: 'github' } : refused(sent, 'github');
	},
	hevy: async (event) => {
		const key = field(await event.request.formData(), 'api_key');
		if (!key) return fail(400, { app: 'hevy', message: m.connections_key_needed() });
		const sent = await connections.connectHevy(event, key);
		return sent.ok ? { connected: 'hevy' } : refused(sent, 'hevy');
	},
	disconnect: async (event) => {
		const app = field(await event.request.formData(), 'app');
		const call = app === 'google' ? connections.disconnectGoogle : app === 'github' ? connections.disconnectGithub : app === 'hevy' ? connections.disconnectHevy : null;
		if (!call) return fail(400, { app, message: m.action_failed() });
		const sent = await call(event);
		return sent.ok ? { disconnected: app } : refused(sent, app);
	},
};
