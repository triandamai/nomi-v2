import { error } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

// Admin pages that work in place (support, integrations, maintenance, roles, model switches) call the
// backend's /api/admin/* through here.
const UUID = '[0-9a-f-]{36}';
const ID = '[A-Za-z0-9_-]{1,64}';
const SLUG = '[a-z0-9][a-z0-9-]{0,62}';
const SECRET = '[A-Z_][A-Z0-9_]{0,127}';
const ALLOWED = new RegExp(
	'^(' +
		[
			'support/articles',
			`support/articles/${UUID}`,
			'support/drafts',
			`support/drafts/${UUID}/(publish|dismiss)`,
			'support/keys',
			`support/keys/${UUID}`,
			'support/questions',
			'shipyard',
			'shipyard/webhook',
			'shipyard/apps',
			'shipyard/apps/link',
			'shipyard/webhooks',
			`shipyard/webhooks/${ID}(/(rotate|test|deliveries))?`,
			`shipyard/deliveries/${ID}/redeliver`,
			'shipyard/orgs',
			`shipyard/orgs/${UUID}(/sync)?`,
			`shipyard/orgs/${UUID}/apps/${SLUG}(/secrets(/${SECRET})?|/envs/${SLUG}(/duplicate)?)?`,
			'shipyard/events',
			'shipyard/events/catch-up',
			`users/${UUID}/admin-access`,
			'permissions/catalog',
			'maintenance',
			`settings/llm/models/${UUID}/enabled`,
		].join('|') +
		')$',
);

const proxy: RequestHandler = async ({ params, request, url, cookies, fetch }) => {
	if (!ALLOWED.test(params.rest)) throw error(404, 'Not found');
	const init: RequestInit = { method: request.method };
	if (request.method !== 'GET' && request.method !== 'HEAD') init.body = await request.text();
	const response = await apiFetch(fetch, cookies, `/api/admin/${params.rest}${url.search}`, init);
	return new Response(response.body, {
		status: response.status,
		headers: { 'content-type': response.headers.get('content-type') ?? 'application/json', 'cache-control': 'no-store' },
	});
};

export const GET = proxy;
export const POST = proxy;
export const PUT = proxy;
export const DELETE = proxy;
export const PATCH = proxy;
