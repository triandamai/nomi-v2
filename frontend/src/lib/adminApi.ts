// Admin pages call the backend's /api/admin/* through routes/admin/(protected)/api.

export async function adminApi<T>(path: string, init?: { method?: string; body?: unknown }): Promise<T> {
	const response = await fetch(`/admin/api/${path}`, {
		method: init?.method ?? 'GET',
		headers: init?.body === undefined ? undefined : { 'content-type': 'application/json' },
		body: init?.body === undefined ? undefined : JSON.stringify(init.body),
	});
	const text = await response.text();
	if (!response.ok) throw new Error(text.trim() || response.statusText);
	return (text ? JSON.parse(text) : null) as T;
}
