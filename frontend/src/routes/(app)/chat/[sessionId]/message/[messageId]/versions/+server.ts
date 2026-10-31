import { error, json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { renderMarkdown } from '$lib/server/markdown';
import { m } from '$lib/paraglide/messages';
import type { MessageVersion } from '$lib/types';
import type { RequestHandler } from './$types';

type ApiVersion = Omit<MessageVersion, 'replies'> & { replies: Omit<MessageVersion['replies'][number], 'content_html'>[] };

// The earlier takes of a turn, their replies rendered as the thread renders them.
export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${encodeURIComponent(params.sessionId)}/messages/${encodeURIComponent(params.messageId)}/versions`);
	if (!response.ok) throw error(response.status, m.msg_versions_failed());
	const body = (await response.json()) as { message_id: string; versions: ApiVersion[]; total: number };
	const versions: MessageVersion[] = await Promise.all(
		body.versions.map(async (version) => ({
			...version,
			replies: await Promise.all(version.replies.map(async (reply) => ({ ...reply, content_html: await renderMarkdown(reply.content) }))),
		})),
	);
	return json({ ...body, versions });
};
