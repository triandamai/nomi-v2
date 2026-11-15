import { json } from '@sveltejs/kit';
import { editRow } from '$lib/server/workspace';
import type { RequestHandler } from './$types';

const isRecord = (value: unknown): value is Record<string, unknown> => typeof value === 'object' && value !== null && !Array.isArray(value);

/**
 * A cell edited in the Database sheet: `{ source, schema, table, key, changes }`, the key the row's
 * primary key. The backend checks it all against the catalog; anything else is refused here.
 */
export const POST: RequestHandler = async (event) => {
	const body: unknown = await event.request.json().catch(() => null);
	if (!isRecord(body) || typeof body.schema !== 'string' || typeof body.table !== 'string' || !isRecord(body.key) || !isRecord(body.changes)) return json({ error: 'bad_request' }, { status: 400 });
	const source = body.source === 'production' ? 'production' : 'development';
	const done = await editRow(event, event.params.id, source, { schema: body.schema, name: body.table }, body.key, body.changes);
	return done.ok ? json({ row: done.row }) : json({ error: done.message }, { status: done.status });
};
