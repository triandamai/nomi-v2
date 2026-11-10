import { proxyFile } from '$lib/server/fileProxy';
import type { RequestHandler } from './$types';

// A recording the crew made with the speak tool (linked as APP_URL/voice/:user/:file).
export const GET: RequestHandler = (event) =>
	proxyFile(event, `/api/voice/${encodeURIComponent(event.params.userId)}/${encodeURIComponent(event.params.file)}`, { fallbackType: 'audio/mpeg' });
