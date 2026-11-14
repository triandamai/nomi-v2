import { error, fail, redirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import { loadChatPage } from '$lib/server/chatPage';
import * as tools from '$lib/server/workspace';
import { changeDomain, loadDeploy, loadDomain, loadProject, publish, rollBack, takeDown, transfer } from '$lib/server/workspace';
import { deleteProject } from '$lib/server/projects';
import { usesSandbox } from '$lib/projects/sandbox';
import { sealPreview } from '$lib/server/previewGrant';
import type { Actions, PageServerLoad } from './$types';

/**
 * A project's workspace (Flow / Build): the project, where it's live, and its chat. Whether it runs
 * in the sandbox is all the page is told here: the sandbox's preview address (a secret) is read by
 * the page's runtime and never written into the page.
 */
export const load: PageServerLoad = async (event) => {
	event.depends('app:workspace-project');
	const loaded = await loadProject(event, event.params.id);
	if (!loaded.project) error(loaded.status === 404 || loaded.status === 403 ? 404 : 503, { message: loaded.status === 404 || loaded.status === 403 ? m.workspace_project_missing() : m.error_title() });
	const id = event.params.id;
	const [chat, deploy, address, people, secrets, showcase, sandbox] = await Promise.all([
		loadChatPage(event, loaded.project.session_id),
		loadDeploy(event, id),
		loadDomain(event, id),
		tools.loadPeople(event, id),
		tools.loadSecrets(event, id),
		tools.loadShowcase(event, id),
		tools.loadSandbox(event, id),
	]);
	// A static site shows its own files, sandboxed, through a grant made for this page (previewGrant.ts).
	const token = event.locals.accessToken;
	const preview = loaded.project.stack === 'static' && loaded.project.files.some((file) => file.path === 'index.html') && token ? `/projects/${id}/preview/~${sealPreview(id, token)}/` : null;
	return { ...chat, preview, title: loaded.project.name, project: loaded.project, live: loaded.live, domain: loaded.domain, deploy, address, people, secrets, showcase, sandbox: usesSandbox(sandbox) };
};

const refused = (sent: { status: number; message: string }) => fail(sent.status, { message: sent.message || m.action_failed() });

export const actions: Actions = {
	/** Sends what's in the preview live, as the next deploy; it runs on its own. */
	publish: async (event) => {
		const sent = await publish(event, event.params.id);
		return sent.ok ? { publishing: true } : refused(sent);
	},
	/** Moves the live app to another name under Nomi's apps domain. */
	domain: async (event) => {
		const name = String((await event.request.formData()).get('name') ?? '').trim().toLowerCase();
		if (!name) return fail(400, { message: m.domain_name_needed() });
		const changed = await changeDomain(event, event.params.id, name);
		if (changed.ok) return { url: changed.url };
		return fail(changed.status, { message: changed.message || m.action_failed(), suggestions: changed.answer?.suggestions ?? [] });
	},
	/** Gives the project to another workspace; the name, typed, confirms it. */
	transfer: async (event) => {
		const form = await event.request.formData();
		const [to, name] = [String(form.get('to') ?? ''), String(form.get('name') ?? '').trim()];
		if (!to) return fail(400, { message: m.transfer_pick() });
		const sent = await transfer(event, event.params.id, to, name);
		if (!sent.ok) return refused(sent);
		redirect(303, `/projects?moved=${encodeURIComponent(name)}`);
	},
	/** Stops serving the live app; publishing brings it back. */
	takedown: async (event) => {
		const sent = await takeDown(event, event.params.id);
		return sent.ok ? { takenDown: true } : refused(sent);
	},
	/** Deletes the project for good, once its name is typed; Projects says so. */
	delete: async (event) => {
		const name = String((await event.request.formData()).get('name') ?? '').trim();
		const loaded = await loadProject(event, event.params.id);
		if (!loaded.project) return fail(404, { message: m.workspace_project_missing() });
		if (name !== loaded.project.name) return fail(400, { field: 'name', message: m.delete_name_mismatch() });
		const deleted = await deleteProject(event, event.params.id);
		if (!deleted.ok) return refused(deleted);
		redirect(303, `/projects?deleted=${encodeURIComponent(name)}`);
	},
	/** People (step 13): invite an address, change someone's role, remove them, or who can open it. */
	invite: async (event) => {
		const form = await event.request.formData();
		const email = String(form.get('email') ?? '').trim().toLowerCase();
		const role = String(form.get('role') ?? 'build');
		if (!/^[^\s@]+@[^\s@]+\.[^\s@.]+$/.test(email)) return fail(400, { field: 'email', message: m.people_email_needed() });
		if (role !== 'build' && role !== 'prompt' && role !== 'view') return fail(400, { message: m.action_failed() });
		const sent = await tools.invitePerson(event, event.params.id, email, role);
		return sent.ok ? { invited: email } : refused(sent);
	},
	role: async (event) => {
		const form = await event.request.formData();
		const [user, role] = [String(form.get('user') ?? ''), String(form.get('role') ?? '')];
		if (!user || (role !== 'admin' && role !== 'member' && role !== 'viewer')) return fail(400, { message: m.action_failed() });
		const sent = await tools.setPersonRole(event, event.params.id, user, role);
		return sent.ok ? { changed: user } : refused(sent);
	},
	remove: async (event) => {
		const user = String((await event.request.formData()).get('user') ?? '');
		if (!user) return fail(400, { message: m.action_failed() });
		const sent = await tools.removePerson(event, event.params.id, user);
		return sent.ok ? { removed: user } : refused(sent);
	},
	access: async (event) => {
		const access = String((await event.request.formData()).get('access') ?? '');
		if (access !== 'org' && access !== 'members') return fail(400, { message: m.action_failed() });
		const sent = await tools.setAccess(event, event.params.id, access);
		return sent.ok ? { access } : refused(sent);
	},
	/** Environment (step 14): a variable set or replaced (the app gets it on its next publish), or removed. */
	secret: async (event) => {
		const form = await event.request.formData();
		const [name, value] = [String(form.get('name') ?? '').trim().toUpperCase(), String(form.get('value') ?? '')];
		if (!/^[A-Z][A-Z0-9_]*$/.test(name)) return fail(400, { field: 'name', message: m.env_name_rule() });
		if (!value) return fail(400, { field: 'value', message: m.env_value_needed() });
		const sent = await tools.setSecret(event, event.params.id, name, value);
		return sent.ok ? { saved: name } : fail(sent.status, { field: 'value', message: sent.message || m.action_failed() });
	},
	unsecret: async (event) => {
		const name = String((await event.request.formData()).get('name') ?? '');
		if (!name) return fail(400, { message: m.action_failed() });
		const sent = await tools.removeSecret(event, event.params.id, name);
		return sent.ok ? { removed: name } : refused(sent);
	},
	/** Export (step 8): one commit of the files to `owner/name`, made when asked, with the person's GitHub. */
	github: async (event) => {
		const form = await event.request.formData();
		const repo = String(form.get('repo') ?? '').trim();
		if (!/^([\w.-]+\/[\w.-]+|https:\/\/github\.com\/[\w.-]+\/[\w.-]+\/?)$/.test(repo)) return fail(400, { field: 'repo', message: m.github_repo_rule() });
		const sent = await tools.pushToGithub(event, event.params.id, { repo, create: form.get('create') === 'true', private: form.get('private') === 'true' });
		return sent.ok ? { pushed: repo } : fail(sent.status, { field: 'repo', message: sent.message || m.action_failed() });
	},
	/** Share (step 7): the live app on show in the community, or only to those with the link. */
	showcase: async (event) => {
		const shown = (await event.request.formData()).get('showcase') === 'true';
		const sent = await tools.setShowcase(event, event.params.id, shown, null);
		return sent.ok ? { showcase: shown } : refused(sent);
	},
	/** Publishes an earlier deploy again, as a new one. */
	rollback: async (event) => {
		const id = String((await event.request.formData()).get('publish') ?? '');
		if (!id) return fail(400, { message: m.action_failed() });
		const sent = await rollBack(event, event.params.id, id);
		return sent.ok ? { rolledBack: id } : refused(sent);
	},
};
