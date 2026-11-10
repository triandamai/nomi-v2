import type { RequestHandler } from './$types';

/** Whether the server is up: 200 to anyone, signed in or not (`/` sends a signed-out visitor to
 * sign in). The e2e runs wait on it; a load balancer can too. */
export const GET: RequestHandler = () => new Response('ok', { headers: { 'content-type': 'text/plain', 'cache-control': 'no-store' } });
