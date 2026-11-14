import { describe, expect, it } from 'vitest';
import { readApiError } from './apiError';

describe('readApiError', () => {
	it('keeps what an emailed-code error says about tries and waiting', async () => {
		expect(await readApiError(Response.json({ error: 'wrong_code', attempts_left: 2 }, { status: 400 }))).toEqual({ status: 400, code: 'wrong_code', message: 'wrong_code', attemptsLeft: 2 });
		expect(await readApiError(Response.json({ error: 'wait', seconds: 42 }, { status: 429 }))).toEqual({ status: 429, code: 'wait', message: 'wait', seconds: 42 });
	});

	it('reads problem details, plain JSON sentences and plain text', async () => {
		const problem = new Response(JSON.stringify({ type: 'about:blank', title: 'Conflict', status: 409, code: 'conflict', detail: 'email already registered' }), {
			status: 409,
			headers: { 'content-type': 'application/problem+json' },
		});
		expect(await readApiError(problem)).toEqual({ status: 409, code: 'conflict', message: 'email already registered' });
		expect(await readApiError(Response.json({ error: 'invalid credentials' }, { status: 401 }))).toEqual({ status: 401, message: 'invalid credentials' });
		expect(await readApiError(new Response('email already registered', { status: 409 }))).toEqual({ status: 409, message: 'email already registered' });
	});

	it('prefers the reason to the code when an error carries both, and keeps the models a connection is used by', async () => {
		expect(await readApiError(Response.json({ error: 'key_rejected', detail: 'openai returned 401 Unauthorized' }, { status: 400 }))).toEqual({
			status: 400,
			code: 'key_rejected',
			message: 'openai returned 401 Unauthorized',
		});
		expect(await readApiError(Response.json({ error: 'not_found', detail: null }, { status: 404 }))).toEqual({ status: 404, code: 'not_found', message: 'not_found' });
		expect(await readApiError(Response.json({ error: 'in_use', models: ['Claude', 'GPT'] }, { status: 409 }))).toEqual({ status: 409, code: 'in_use', message: 'in_use', models: ['Claude', 'GPT'] });
	});

	it('says something even when the body is empty', async () => {
		expect(await readApiError(new Response(null, { status: 502 }))).toEqual({ status: 502, message: '' });
	});
});
