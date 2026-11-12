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
