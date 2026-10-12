// The open project's sheets talk to the backend through the project's own proxy
// (routes/(app)/projects/[projectId]/api), so the page never leaves and the preview keeps running.

export class ProjectApiError extends Error {
	constructor(
		readonly status: number,
		message: string,
	) {
		super(message);
	}
}

/** Calls `/api/projects/:id/<path>` and returns its JSON (null for an empty reply). */
export async function projectApi<T>(projectId: string, path: string, init?: { method?: string; body?: unknown }): Promise<T> {
	const response = await fetch(`/projects/${projectId}/api${path ? `/${path}` : ''}`, {
		method: init?.method ?? 'GET',
		headers: init?.body === undefined ? undefined : { 'content-type': 'application/json' },
		body: init?.body === undefined ? undefined : JSON.stringify(init.body),
	});
	const text = await response.text();
	if (!response.ok) {
		let message = text;
		try {
			const parsed = JSON.parse(text) as { message?: string; error?: string };
			message = parsed.message ?? parsed.error ?? text;
		} catch {
			// Plain text from the backend: shown as is.
		}
		throw new ProjectApiError(response.status, message.trim() || response.statusText);
	}
	return (text ? JSON.parse(text) : null) as T;
}

/** "3 min ago", "yesterday", or a date. */
export function ago(at: string, locale: string): string {
	const then = new Date(at).getTime();
	const seconds = Math.round((Date.now() - then) / 1000);
	const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' });
	if (seconds < 60) return rtf.format(-seconds, 'second');
	if (seconds < 3600) return rtf.format(-Math.round(seconds / 60), 'minute');
	if (seconds < 86_400) return rtf.format(-Math.round(seconds / 3600), 'hour');
	if (seconds < 7 * 86_400) return rtf.format(-Math.round(seconds / 86_400), 'day');
	return new Intl.DateTimeFormat(locale, { dateStyle: 'medium' }).format(new Date(at));
}

/** Bytes as KB, MB or GB. */
export function bytes(value: number, locale: string): string {
	const units = ['B', 'KB', 'MB', 'GB', 'TB'];
	let n = value;
	let unit = 0;
	while (n >= 1024 && unit < units.length - 1) {
		n /= 1024;
		unit += 1;
	}
	return `${new Intl.NumberFormat(locale, { maximumFractionDigits: n < 10 && unit > 0 ? 1 : 0 }).format(n)} ${units[unit]}`;
}
