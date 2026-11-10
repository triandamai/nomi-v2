/**
 * What a failed backend call said, whichever way it said it: a JSON `{ error, ... }` (the emailed
 * code's errors carry `attempts_left` or `seconds`), RFC 9457 problem details (`code`, `detail`),
 * or a plain sentence. Auth-code calls aren't sent `Accept: application/problem+json`: that
 * rewrite drops the extra fields.
 */
export type ApiError = { status: number; code?: string; message: string; attemptsLeft?: number; seconds?: number };

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
	const said = typeof body.error === 'string' ? body.error : typeof body.detail === 'string' ? body.detail : '';
	const code = typeof body.code === 'string' ? body.code : typeof body.error === 'string' && SLUG.test(body.error) ? body.error : undefined;
	return {
		status: response.status,
		...(code && { code }),
		message: said,
		...(typeof body.attempts_left === 'number' && { attemptsLeft: body.attempts_left }),
		...(typeof body.seconds === 'number' && { seconds: body.seconds }),
	};
}
