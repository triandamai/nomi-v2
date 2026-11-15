import { databaseDump } from '$lib/server/workspace';
import type { RequestHandler } from './$types';

/** Download dump: the database asked for (`?source=`) as a `.sql` file, streamed as the backend names it. */
export const GET: RequestHandler = async (event) => {
	const source = event.url.searchParams.get('source') === 'production' ? 'production' : 'development';
	const response = await databaseDump(event, event.params.id, source);
	if (!response.ok) return new Response(null, { status: response.status });
	const headers = new Headers({ 'content-type': response.headers.get('content-type') ?? 'application/sql', 'cache-control': 'no-store' });
	const disposition = response.headers.get('content-disposition');
	if (disposition) headers.set('content-disposition', disposition);
	return new Response(response.body, { status: 200, headers });
};
