import { apiUrl } from './api';

export type Maintenance = { message: string | null; until: string | null };

/**
 * Whether Nomi is down for maintenance (the public `GET /api/maintenance`). A check that fails
 * lets people in: the backend being briefly unreachable isn't maintenance.
 */
export async function getMaintenance(fetchFn: typeof fetch): Promise<Maintenance | null> {
	try {
		const response = await fetchFn(apiUrl('/api/maintenance'));
		if (!response.ok) return null;
		const state = (await response.json()) as { enabled?: boolean; message?: string | null; until?: string | null };
		return state.enabled ? { message: state.message ?? null, until: state.until ?? null } : null;
	} catch {
		return null;
	}
}
