import { proxyFile } from '$lib/server/fileProxy';
import type { RequestHandler } from './$types';

// A profile picture (the backend links them as APP_URL/avatars/:user/:file). Someone signed out
// sees the public copy, as on a shared chat.
export const GET: RequestHandler = (event) => {
	const path = `${encodeURIComponent(event.params.userId)}/${encodeURIComponent(event.params.file)}`;
	return proxyFile(event, `/api/avatars/${path}`, { publicPath: `/api/public/avatars/${path}`, fallbackType: 'application/octet-stream' });
};
