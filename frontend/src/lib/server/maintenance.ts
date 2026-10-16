import { apiUrl } from '$lib/server/api';

export interface MaintenanceState {
	enabled: boolean;
	message: string;
	until: string | null;
}

const OFF: MaintenanceState = { enabled: false, message: '', until: null };
const FRESH_MS = 10_000;
let cached: { at: number; state: MaintenanceState } | null = null;

/** Whether maintenance mode is on (asked of the API at most every 10 seconds). When the API
 * can't be reached the app carries on: its own pages will say what's wrong. */
export async function maintenanceState(fetchFn: typeof fetch = fetch): Promise<MaintenanceState> {
	if (cached && Date.now() - cached.at < FRESH_MS) return cached.state;
	try {
		const response = await fetchFn(apiUrl('/api/maintenance'));
		const state = response.ok ? ((await response.json()) as MaintenanceState) : OFF;
		cached = { at: Date.now(), state };
		return state;
	} catch {
		return cached?.state ?? OFF;
	}
}
