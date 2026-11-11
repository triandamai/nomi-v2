import postgres from 'postgres';
import { DATABASE_URL } from './env';

// Straight to the e2e database, for what no API can do: making someone a platform admin
// (`users.is_platform_admin` is set only in the database), and putting back shared settings a
// test changed (models and custom agents have no delete the test could use).

async function withDb<T>(run: (sql: postgres.Sql) => Promise<T>): Promise<T> {
	const sql = postgres(DATABASE_URL, { max: 1, onnotice: () => {} });
	try {
		return await run(sql);
	} finally {
		await sql.end();
	}
}

/** Makes the account with this email a platform admin: every admin area at manage, from its next token. */
export async function makePlatformAdmin(email: string): Promise<void> {
	const updated = await withDb((sql) => sql`UPDATE users SET is_platform_admin = true WHERE id = (SELECT user_id FROM web_credentials WHERE email = ${email})`);
	if (updated.count !== 1) throw new Error(`No account for ${email}`);
}

/** Removes an admin model a test added (deleting the default one is refused by the API). */
export async function deleteAdminModel(id: string): Promise<void> {
	await withDb((sql) => sql`DELETE FROM admin_llm_models WHERE id = ${id}`);
}

/** Removes a custom agent a test added (the API has no delete). */
export async function deleteDynamicAgent(id: string): Promise<void> {
	await withDb((sql) => sql`DELETE FROM dynamic_agents WHERE id = ${id}`);
}
