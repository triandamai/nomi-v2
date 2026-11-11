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
