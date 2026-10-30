import { error, fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { retryRun } from '$lib/server/retryRun';
import { loadModelOptions, loadSessionModel, sessionModelActions } from '$lib/server/modelOptions';
import { fetchMessagePage } from '$lib/server/fetchMessagePage';
import { renameSession } from '$lib/server/renameSession';
import { moveSession, pinSession } from '$lib/server/updateSession';
import { loadThinkingLevel, saveThinkingLevel } from '$lib/server/thinking';
import type { AgentStatus, ChatPersonality, MessageItem, OrgList, PersonalityHistoryResponse, ProjectList, RenderedMessage, SessionSummary } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
	const firstPage = await fetchMessagePage(fetch, cookies, params.sessionId);
	if (firstPage.status === 404) {
		throw redirect(303, '/');
	}
	if (!firstPage.messages) {
		throw error(firstPage.status, m.err_load_chat());
	}
	const messages: RenderedMessage[] = firstPage.messages;
	const hasOlder = firstPage.hasOlder;

	const personalityResponse = await apiFetch(fetch, cookies, '/api/personality/history');
	const personality: PersonalityHistoryResponse = personalityResponse.ok
		? ((await personalityResponse.json()) as PersonalityHistoryResponse)
		: { versions: [] };
	const chatPersonalityResponse = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/personality`);
	const chatPersonality: ChatPersonality = chatPersonalityResponse.ok
		? ((await chatPersonalityResponse.json()) as ChatPersonality)
		: { chat: null, default: null };

	const agentActivityResponse = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/agent-activity`);
	const agentActivity = agentActivityResponse.ok ? await agentActivityResponse.json() : [];

	const agentStatusResponse = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/agent-status`);
	const agentStatus: AgentStatus | null = agentStatusResponse.ok ? await agentStatusResponse.json() : null;

	// There's no single-session endpoint; the title comes from the session list (best-effort).
	const sessionsResponse = await apiFetch(fetch, cookies, '/api/sessions');
	const sessions: SessionSummary[] = sessionsResponse.ok
		? ((await sessionsResponse.json()) as { sessions: SessionSummary[] }).sessions
		: [];
	const session = sessions.find((s) => s.id === params.sessionId);
	const title = session?.title ?? m.nav_new_chat();
	// The chat menu: whether it's pinned, the project it's in, and the projects it can move to
	// (named with their workspace). Best-effort: without them the menu still renames and deletes.
	const [projectsResponse, orgsResponse] = await Promise.all([apiFetch(fetch, cookies, '/api/projects'), apiFetch(fetch, cookies, '/api/orgs')]);
	const projectList = projectsResponse.ok ? ((await projectsResponse.json()) as ProjectList) : null;
	const orgs = orgsResponse.ok ? ((await orgsResponse.json()) as OrgList).orgs : [];
	const workspace = orgs.find((org) => org.id === projectList?.org_id)?.name ?? '';
	const chatMenu = {
		pinned: session?.pinned ?? false,
		inProjectId: session?.in_project_id ?? null,
		projects: (projectList?.projects ?? []).map(({ id, name }) => ({ id, name, workspace })),
	};

	// The composer's model picker: what's on offer, and what this chat uses. Neither can break
	// the chat; the provider catalog loads only if the person connects one from there.
	const [thinkingLevel, modelOptions, sessionModel] = await Promise.all([
		loadThinkingLevel(fetch, cookies, params.sessionId),
		loadModelOptions(fetch, cookies),
		loadSessionModel(fetch, cookies, params.sessionId),
	]);
	return { messages, hasOlder, personality, chatPersonality, agentActivity, agentStatus, title, thinkingLevel, modelOptions, sessionModel, chatMenu };
};

export const actions: Actions = {
	rename: async ({ request, params, cookies, fetch }) => renameSession(fetch, cookies, params.sessionId, await request.formData()),
	pinSession: async ({ request, params, cookies, fetch }) => pinSession(fetch, cookies, params.sessionId, await request.formData()),
	moveSession: async ({ request, params, cookies, fetch }) => moveSession(fetch, cookies, params.sessionId, await request.formData()),
	deleteSession: async ({ params, cookies, fetch }) => {
		const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}`, { method: 'DELETE' });
		if (!response.ok) return fail(response.status, { error: m.err_delete_chat() });
		return { deleted: true };
	},
	setThinking: async ({ request, params, cookies, fetch }) => saveThinkingLevel(fetch, cookies, params.sessionId, await request.formData()),

	// Named (not `default`) because this actions object also has the model picker's and the rest —
	// SvelteKit forbids mixing a `default` action with named actions in the
	// same file (throws "When using named actions, the default action cannot be used" at request
	// time), so the message-send form below must target this action explicitly.
	sendMessage: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const text = data.get('text');

		if (typeof text !== 'string' || !text.trim()) {
			return fail(400, { error: m.err_empty_message() });
		}

		const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/messages`, {
			method: 'POST',
			body: JSON.stringify({ text }),
		});

		if (response.status === 404) {
			throw redirect(303, '/');
		}
		if (!response.ok) {
			return fail(response.status, { error: m.err_send_message() });
		}

		const { user_message, supervisor_reply } = (await response.json()) as {
			user_message: MessageItem;
			supervisor_reply?: MessageItem;
		};

		// A stop command is answered on the spot by the supervisor, so no turn is coming.
		return { user_message, stopped: supervisor_reply !== undefined };
	},

	// The composer's model picker: this chat's model, which models it offers, and connecting a
	// provider in place from Manage models.
	...sessionModelActions,

	feedback: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const messageId = data.get('messageId');
		const rating = data.get('rating');
		// A thumbs-down's reasons, note and whether the chat may be read (DislikeFeedback).
		const reasons = data.getAll('reasons').filter((reason): reason is string => typeof reason === 'string' && reason !== '');
		const note = String(data.get('note') ?? '').trim();
		const includeChat = data.get('includeChat') === 'true';

		if (typeof messageId !== 'string' || !messageId) {
			return fail(400, { error: m.err_invalid_message() });
		}

		const path = `/api/sessions/${params.sessionId}/messages/${messageId}/feedback`;
		const response =
			rating === 'up' || rating === 'down'
				? await apiFetch(fetch, cookies, path, { method: 'PUT', body: JSON.stringify({ rating, reasons, note: note || null, include_chat: includeChat }) })
				: await apiFetch(fetch, cookies, path, { method: 'DELETE' });

		if (!response.ok) {
			return fail(response.status, { error: m.err_save_feedback() });
		}
		return { success: true };
	},

	/** Try again on a failed run's error bubble. */
	retryRun: async ({ request, params, cookies, fetch }) => retryRun(fetch, cookies, params.sessionId, await request.formData()),

	resolveApproval: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const messageId = data.get('messageId');
		const decision = data.get('decision');
		const remember = data.get('remember') === 'true';

		if (typeof messageId !== 'string' || (decision !== 'approve' && decision !== 'deny')) {
			return fail(400, { error: m.err_invalid_approval() });
		}

		const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/messages/${messageId}/approval`, {
			method: 'PUT',
			body: JSON.stringify({ decision, remember }),
		});

		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_record_decision() });
		}

		return { success: true };
	},

	/** A past personality for this chat only, or (scope all_chats) as the default everywhere. */
	usePersonality: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const description = String(data.get('description') ?? '').trim();
		const scope = data.get('scope') === 'all_chats' ? 'all_chats' : 'chat';
		if (!description) return fail(400, { personalityError: m.err_invalid_version() });
		const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/personality`, {
			method: 'PUT',
			body: JSON.stringify({ description, scope }),
		});
		if (!response.ok) return fail(response.status, { personalityError: (await response.text()) || m.err_rollback_personality() });
		return { personalityRestored: true };
	},

	/** This chat goes back to the default personality. */
	resetPersonality: async ({ params, cookies, fetch }) => {
		const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/personality`, { method: 'DELETE' });
		if (!response.ok) return fail(response.status, { personalityError: (await response.text()) || m.err_rollback_personality() });
		return { personalityRestored: true };
	},

	restorePersonality: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const version = data.get('version');

		if (typeof version !== 'string' || !version.trim()) {
			return fail(400, { personalityError: m.err_invalid_version() });
		}

		const response = await apiFetch(fetch, cookies, '/api/personality/rollback', {
			method: 'POST',
			body: JSON.stringify({ version: Number(version) }),
		});

		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { personalityError: message || m.err_rollback_personality() });
		}

		return { personalityRestored: true };
	},
};
