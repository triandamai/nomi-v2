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
export async function seedMemory(email: string, memory: { content: string; kind: string; weight?: number }): Promise<string> {
	const sql = postgres(DATABASE_URL, { max: 1, onnotice: () => {} });
	try {
		const [row] = await sql`
			INSERT INTO memory_items (user_id, content, kind, weight, embedding)
			SELECT user_id, ${memory.content}, ${memory.kind}, ${memory.weight ?? 1}, array_fill(0::real, ARRAY[1536])::vector
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
