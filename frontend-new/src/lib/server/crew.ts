import { isRedirect } from '@sveltejs/kit';
import type { CrewMember, ModelOptions, PersonalityVersion, UserModels } from '$lib/adapters/api';
import { apiFetch, type ApiEvent } from './api';

// Settings › Crew's calls to the backend, as the person: who's on the crew and what each is doing,
// the models Nomi and Koda can work on (set through Models & providers' own actions), and the
// personality's versions, one of which can be brought back.

export type Sent<T> = { ok: true; data: T } | { ok: false; status: number; message: string };

async function call(event: ApiEvent, path: string, init?: RequestInit): Promise<Response> {
	try {
		return await apiFetch(event, path, init);
	} catch (error) {
		if (isRedirect(error)) throw error;
		return new Response('', { status: 503 });
	}
}

async function read<T>(event: ApiEvent, path: string): Promise<T | null> {
	const response = await call(event, path);
	return response.ok ? ((await response.json()) as T) : null;
}

export type CrewData = { members: CrewMember[] | null; options: ModelOptions | null; models: UserModels | null; personality: PersonalityVersion[] | null };

export async function loadCrew(event: ApiEvent): Promise<CrewData> {
	const [agents, options, models, history] = await Promise.all([
		read<{ agents: CrewMember[] }>(event, '/api/agents'),
		read<ModelOptions>(event, '/api/llm/model-options'),
		read<UserModels>(event, '/api/llm/models'),
		read<{ versions: PersonalityVersion[] }>(event, '/api/personality/history'),
	]);
	return { members: agents?.agents ?? null, options, models, personality: history?.versions ?? null };
}

export async function restorePersonality(event: ApiEvent, version: number): Promise<Sent<null>> {
	const response = await call(event, '/api/personality/rollback', { method: 'POST', body: JSON.stringify({ version }) });
	if (response.ok) return { ok: true, data: null };
	return { ok: false, status: response.status, message: (await response.text()).trim().slice(0, 200) };
}
