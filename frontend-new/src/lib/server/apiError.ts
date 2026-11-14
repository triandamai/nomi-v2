/**
 * What a failed backend call said, whichever way it said it: a JSON `{ error, detail?, ... }` (the emailed
 * code's errors carry `attempts_left` or `seconds`), RFC 9457 problem details (`code`, `detail`),
 * or a plain sentence. Auth-code calls aren't sent `Accept: application/problem+json`: that
 * rewrite drops the extra fields.
 */
export type ApiError = { status: number; code?: string; message: string; attemptsLeft?: number; seconds?: number; models?: string[] };

const SLUG = /^[a-z][a-z0-9_]*$/;

export async function readApiError(response: Response): Promise<ApiError> {
	const text = await response.text().catch(() => '');
	const error: ApiError = { status: response.status, message: text.trim() };
	let body: Record<string, unknown> | null = null;
	try {
		body = text ? (JSON.parse(text) as Record<string, unknown>) : null;
	} catch {
		return error;
	}
	if (!body || typeof body !== 'object') return error;
	const code = typeof body.code === 'string' ? body.code : typeof body.error === 'string' && SLUG.test(body.error) ? body.error : undefined;
	// A coded error may carry its reason beside the code (`{ error: 'key_rejected', detail }`).
	const detail = typeof body.detail === 'string' ? body.detail : '';
	const said = typeof body.error === 'string' ? (code && detail ? detail : body.error) : detail;
	return {
		status: response.status,
		...(code && { code }),
		message: said,
		...(typeof body.attempts_left === 'number' && { attemptsLeft: body.attempts_left }),
		...(typeof body.seconds === 'number' && { seconds: body.seconds }),
		...(Array.isArray(body.models) && { models: body.models.filter((model): model is string => typeof model === 'string') }),
	};
}
