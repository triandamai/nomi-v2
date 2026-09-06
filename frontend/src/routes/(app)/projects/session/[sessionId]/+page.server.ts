import { error, fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { renderMarkdown } from '$lib/server/markdown';
import type { MessageItem, ProjectDetail, RenderedMessage } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
	const messagesResponse = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/messages`);
	if (messagesResponse.status === 404) {
		throw redirect(303, '/projects');
	}
	if (!messagesResponse.ok) {
		throw error(messagesResponse.status, 'Could not load this project.');
	}
	const { messages: rawMessages } = (await messagesResponse.json()) as { messages: MessageItem[] };
	const messages: RenderedMessage[] = await Promise.all(
		rawMessages.map(async (message) => ({ ...message, content_html: await renderMarkdown(message.content) })),
	);

	const agentActivityResponse = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/agent-activity`);
	const agentActivity = agentActivityResponse.ok ? await agentActivityResponse.json() : [];

	// A session doesn't have a project yet until the planning agent calls create_project mid-chat
	// — a 404 here is a normal "nothing to build yet" state, not an error.
	const projectResponse = await apiFetch(fetch, cookies, `/api/projects/by-session/${params.sessionId}`);
	const project: ProjectDetail | null = projectResponse.ok ? await projectResponse.json() : null;

	return { messages, agentActivity, project };
};

export const actions: Actions = {
	sendMessage: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const text = data.get('text');

		if (typeof text !== 'string' || !text.trim()) {
			return fail(400, { error: 'Message cannot be empty.' });
		}

		const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/messages`, {
			method: 'POST',
			body: JSON.stringify({ text }),
		});

		if (response.status === 404) {
			throw redirect(303, '/projects');
		}
		if (!response.ok) {
			return fail(response.status, { error: 'Failed to send message.' });
		}

		const { user_message } = (await response.json()) as { user_message: MessageItem };

		return { user_message };
	},

	feedback: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const messageId = data.get('messageId');
		const rating = data.get('rating');

		if (typeof messageId !== 'string' || !messageId) {
			return fail(400, { error: 'Invalid message.' });
		}

		const path = `/api/sessions/${params.sessionId}/messages/${messageId}/feedback`;
		const response =
			rating === 'up' || rating === 'down'
				? await apiFetch(fetch, cookies, path, { method: 'PUT', body: JSON.stringify({ rating }) })
				: await apiFetch(fetch, cookies, path, { method: 'DELETE' });

		if (!response.ok) {
			return fail(response.status, { error: 'Failed to save feedback.' });
		}
		return { success: true };
	},

	loadFile: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const projectId = data.get('projectId');
		const path = data.get('path');
		if (typeof projectId !== 'string' || !projectId || typeof path !== 'string' || !path) {
			return fail(400, { error: 'Invalid path.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/projects/${projectId}/files/${path}`);
		if (!response.ok) {
			return fail(response.status, { error: 'Failed to load file.' });
		}
		const content = await response.text();
		return { success: true, path, content };
	},

	saveFile: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const projectId = data.get('projectId');
		const path = data.get('path');
		const content = data.get('content');
		if (
			typeof projectId !== 'string' ||
			!projectId ||
			typeof path !== 'string' ||
			!path ||
			typeof content !== 'string'
		) {
			return fail(400, { error: 'Invalid file.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/projects/${projectId}/files/${path}`, {
			method: 'PUT',
			body: JSON.stringify({ content }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to save file.' });
		}
		return { success: true };
	},
};
