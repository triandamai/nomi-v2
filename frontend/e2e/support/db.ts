import postgres from 'postgres';

const DATABASE_URL = process.env.DATABASE_URL ?? 'postgres://postgres:postgres@localhost:5432/nomi_dev';

export async function promoteToPlatformAdmin(email: string): Promise<void> {
	const sql = postgres(DATABASE_URL);
	try {
		await sql`UPDATE users SET is_platform_admin = true
			WHERE id = (SELECT user_id FROM web_credentials WHERE email = ${email})`;
	} finally {
		await sql.end();
	}
}
