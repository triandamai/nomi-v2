import postgres from 'postgres';

// Replies the fake model can't write: a reply from the crew with content blocks (clarifying
// questions, tables, views…), put straight into the e2e database, as the crew's turn would. A
// message with no sender identity is the crew's.

const DATABASE_URL = process.env.E2E_DATABASE_URL ?? 'postgres://nomi:nomi@localhost:5432/nomi_e2e_v2';

export async function seedReply(sessionId: string, reply: { content: string; agent?: string; blocks: unknown[] }): Promise<string> {
	const sql = postgres(DATABASE_URL, { max: 1, onnotice: () => {} });
	try {
		const [row] = await sql`
			INSERT INTO messages (session_id, content, content_blocks, agent_display_name)
			VALUES (${sessionId}, ${reply.content}, ${sql.json(reply.blocks as never)}, ${reply.agent ?? 'Nomi'})
			RETURNING id`;
		return row.id as string;
	} finally {
		await sql.end();
	}
}

/** A notification for the person with this email, as the backend's own would land in their inbox. */
export async function seedNotification(email: string, notification: { kind?: string; title: string; body: string; link?: string; read?: boolean }): Promise<string> {
	const sql = postgres(DATABASE_URL, { max: 1, onnotice: () => {} });
	try {
		const [row] = await sql`
			INSERT INTO notifications (user_id, kind, title, body, link, read_at)
			SELECT user_id, ${notification.kind ?? 'account'}, ${notification.title}, ${notification.body}, ${notification.link ?? null}, ${notification.read ? sql`now()` : null}
			FROM web_credentials WHERE email = ${email}
			RETURNING id`;
		return row.id as string;
	} finally {
		await sql.end();
	}
}

/** A reminder that has gone off (the reminders worker's job, which takes until its time). */
export async function fireReminder(id: string): Promise<void> {
	const sql = postgres(DATABASE_URL, { max: 1, onnotice: () => {} });
	try {
		await sql`UPDATE reminders SET status = 'fired', due_at = now() - interval '5 minutes', last_fired_at = now() - interval '5 minutes' WHERE id = ${id}`;
	} finally {
		await sql.end();
	}
}

/** Usage the crew's turns would have recorded: tokens on a model, now (the fake model records none). */
export async function seedUsage(email: string, usage: { model: string; input: number; output: number; source?: 'nomi' | 'own_key'; costUsd?: number }): Promise<void> {
	const sql = postgres(DATABASE_URL, { max: 1, onnotice: () => {} });
	try {
		await sql`
			INSERT INTO llm_usage (user_id, source, model_label, provider, model_id, input_tokens, output_tokens, cost_usd)
			SELECT user_id, ${usage.source ?? 'nomi'}, ${usage.model}, 'fake', ${usage.model}, ${usage.input}, ${usage.output}, ${usage.costUsd ?? 0}
			FROM web_credentials WHERE email = ${email}`;
	} finally {
		await sql.end();
	}
}

/** A memory the crew kept, as its turn would have (a zero vector stands in for the embedding). */
export async function seedMemory(email: string, memory: { content: string; kind: string; weight?: number; learnedBy?: string }): Promise<string> {
	const sql = postgres(DATABASE_URL, { max: 1, onnotice: () => {} });
	try {
		const [row] = await sql`
			INSERT INTO memory_items (user_id, content, kind, weight, embedding, learned_by)
			SELECT user_id, ${memory.content}, ${memory.kind}, ${memory.weight ?? 1}, array_fill(0::real, ARRAY[1536])::vector, ${memory.learnedBy ?? null}
			FROM web_credentials WHERE email = ${email}
			RETURNING id`;
		return row.id as string;
	} finally {
		await sql.end();
	}
}

/**
 * One of Nomi's own models (the fake one), as an admin would add it, unless there's one already:
 * the Included group on Models & providers lists them. A fresh database has none.
 */
export async function seedNomiModel(email: string): Promise<void> {
	const sql = postgres(DATABASE_URL, { max: 1, onnotice: () => {} });
	try {
		await sql`
			INSERT INTO admin_llm_models (label, provider, model_id, api_key_encrypted, is_default, updated_by)
			SELECT 'E2E default (fake)', 'fake', 'fake', ''::bytea, NOT EXISTS (SELECT 1 FROM admin_llm_models), user_id
			FROM web_credentials WHERE email = ${email}
			AND NOT EXISTS (SELECT 1 FROM admin_llm_models WHERE enabled)`;
	} finally {
		await sql.end();
	}
}

/** Versions of how Nomi speaks, oldest first, the last one in use, as its personality changes would leave them. */
export async function seedPersonality(email: string, descriptions: string[]): Promise<void> {
	const sql = postgres(DATABASE_URL, { max: 1, onnotice: () => {} });
	try {
		const [person] = await sql`SELECT user_id FROM web_credentials WHERE email = ${email}`;
		const current = descriptions.length;
		await sql`
			INSERT INTO user_personality (user_id, description, current_version) VALUES (${person.user_id}, ${descriptions[current - 1]}, ${current})
			ON CONFLICT (user_id) DO UPDATE SET description = EXCLUDED.description, current_version = EXCLUDED.current_version`;
		for (const [index, description] of descriptions.entries()) {
			await sql`INSERT INTO user_personality_versions (user_id, version, description) VALUES (${person.user_id}, ${index + 1}, ${description})`;
		}
	} finally {
		await sql.end();
	}
}

/** A project's state as Koda's build leaves it (`building` once files are written, `ready` when it reports done). */
export async function setProjectStatus(id: string, status: 'planning' | 'building' | 'ready'): Promise<void> {
	const sql = postgres(DATABASE_URL, { max: 1, onnotice: () => {} });
	try {
		await sql`UPDATE projects SET status = ${status} WHERE id = ${id}`;
	} finally {
		await sql.end();
	}
}

/** Marks a project as published on Shipyard (its app named), as a first publish would. */
export async function markPublished(projectId: string): Promise<void> {
	const sql = postgres(DATABASE_URL, { max: 1, onnotice: () => {} });
	try {
		await sql`UPDATE projects SET shipyard_app = ${`e2e-${projectId.slice(0, 8)}`} WHERE id = ${projectId}`;
	} finally {
		await sql.end();
	}
}

/** What a published site has reported about itself: its visits (with where from) and forms sent. */
export async function siteHits(projectId: string): Promise<{ kind: string; referrer_host: string | null }[]> {
	const sql = postgres(DATABASE_URL, { max: 1, onnotice: () => {} });
	try {
		return (await sql`SELECT kind, referrer_host FROM site_hits WHERE project_id = ${projectId} ORDER BY id`) as unknown as { kind: string; referrer_host: string | null }[];
	} finally {
		await sql.end();
	}
}

/**
 * A publish of a project as Shipyard's webhook leaves it (Mobile Build — Workspace tools, step 4):
 * live or failed, with the failure's step, message and log where it failed. No Shipyard runs in
 * e2e, so a publish can't be started from the app; the deploy overview reads these rows as it is.
 */
export async function seedPublish(projectId: string, publish: { version: number; status: 'succeeded' | 'failed' | 'running'; url?: string; error?: string; failure?: Record<string, unknown>; minutesAgo?: number }): Promise<string> {
	const sql = postgres(DATABASE_URL, { max: 1, onnotice: () => {} });
	try {
		const at = new Date(Date.now() - (publish.minutesAgo ?? 0) * 60_000);
		const finished = publish.status === 'running' ? null : new Date(at.getTime() + 41_000);
		const [row] = await sql`
			INSERT INTO project_publishes (project_id, environment, status, url, error, manifest, files_version, version, created_at, finished_at, failure)
			VALUES (${projectId}, 'production', ${publish.status}, ${publish.url ?? null}, ${publish.error ?? null}, ${sql.json({})}, ${publish.version}, ${publish.version}, ${at}, ${finished}, ${publish.failure ? sql.json(publish.failure as never) : null})
			RETURNING id`;
		return row.id as string;
	} finally {
		await sql.end();
	}
}

/** A plan a crew member wrote in a chat, one row per version (its markdown checklist), as the planning turn keeps them. */
export async function seedPlan(sessionId: string, email: string, plan: { title: string; versions: string[] }): Promise<{ agentSessionId: string; planId: string }> {
	const sql = postgres(DATABASE_URL, { max: 1, onnotice: () => {} });
	try {
		const agentSessionId = crypto.randomUUID();
		let planId = '';
		for (const [index, content] of plan.versions.entries()) {
			const [row] = await sql`
				INSERT INTO agent_plans (session_id, agent_session_id, user_id, title, content, version)
				SELECT ${sessionId}, ${agentSessionId}, user_id, ${plan.title}, ${content}, ${index + 1}
				FROM web_credentials WHERE email = ${email}
				RETURNING id`;
			planId = row.id as string;
		}
		return { agentSessionId, planId };
	} finally {
		await sql.end();
	}
}

/** A crew session paused on an approval in a message, as the crew's turn leaves it (so deciding it is accepted). */
export async function seedPendingApproval(sessionId: string, email: string, messageId: string, agentType = 'workspace'): Promise<void> {
	const sql = postgres(DATABASE_URL, { max: 1, onnotice: () => {} });
	try {
		await sql`
			INSERT INTO agent_sessions (session_id, sender_channel_identity_id, agent_type, status, state)
			SELECT ${sessionId}, ci.id, ${agentType}, 'awaiting_approval', ${sql.json({ pending_approval_message_id: messageId, paused_for_approval: true })}
			FROM channel_identities ci JOIN web_credentials w ON w.user_id = ci.user_id
			WHERE w.email = ${email} LIMIT 1`;
	} finally {
		await sql.end();
	}
}
