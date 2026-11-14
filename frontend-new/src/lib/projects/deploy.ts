import { m } from '$lib/paraglide/messages';
import type { DeployOverview, PublishRow } from '$lib/adapters/api';

// Publishing a project (Flow / Build — Workspace tools U16wU, steps 2–7): what the Publish button
// says, from the backend's deploy overview, and the Deployments sheet's rows. Changes stay in the
// preview until Publish; the count is what goes live. A failed publish keeps the last live deploy
// running. Rolling back is a publish too, so it's always reversible.

export type PublishState = { state: 'off' } | { state: 'idle' } | { state: 'ready'; count: number } | { state: 'publishing'; elapsed: string } | { state: 'failed' };

const elapsed = (from: string, now: Date) => {
	const seconds = Math.max(0, Math.round((now.getTime() - Date.parse(from)) / 1000));
	return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`;
};

/** Production's publishes, newest first. */
const production = (overview: DeployOverview) => overview.publishes.filter((one) => one.environment === 'production');

/** `files` is how many files the project has, for the count before its first publish. */
export function publishState(overview: DeployOverview, files: number, now: Date): PublishState {
	if (!overview.available) return { state: 'off' };
	const [newest] = production(overview);
	if (newest?.status === 'running') return { state: 'publishing', elapsed: elapsed(newest.created_at, now) };
	if (overview.backend_setup) return { state: 'publishing', elapsed: '' };
	if (newest?.status === 'failed') return { state: 'failed' };
	const waiting = overview.unpublished_changes ?? (production(overview).some((one) => one.status === 'succeeded') ? 0 : files);
	return waiting > 0 ? { state: 'ready', count: waiting } : { state: 'idle' };
}

/** While a publish runs (or its backend is being set up), the workspace keeps looking. */
export function publishing(overview: DeployOverview): boolean {
	return Boolean(overview.backend_setup) || production(overview).some((one) => one.status === 'running');
}

export type DeployRowData = {
	id: string;
	number: string;
	title: string;
	meta: string;
	state: 'live' | 'running' | 'failed' | 'rollback' | 'kept';
	error?: string;
	version: number;
};

function when(iso: string, now: Date, timeZone?: string): string {
	const at = new Date(iso);
	const day = (date: Date) => new Intl.DateTimeFormat('en-CA', { timeZone, year: 'numeric', month: '2-digit', day: '2-digit' }).format(date);
	const time = new Intl.DateTimeFormat('en-GB', { timeZone, hour: '2-digit', minute: '2-digit', hour12: false }).format(at);
	if (day(at) === day(now)) return m.deploy_today({ time });
	if (day(at) === day(new Date(now.getTime() - 86_400_000))) return m.deploy_yesterday({ time });
	return m.deploy_on({ time, date: new Intl.DateTimeFormat('en-GB', { timeZone, day: 'numeric', month: 'short' }).format(at) });
}

const titleCase = (name: string) => name.charAt(0).toUpperCase() + name.slice(1);

export function deployRows(publishes: PublishRow[], now: Date, timeZone?: string): DeployRowData[] {
	const first = Math.min(...publishes.filter((one) => one.status === 'succeeded').map((one) => one.version));
	return publishes.map((one) => {
		const who = one.started_by_agent ? titleCase(one.started_by_agent) : one.created_by;
		const title = one.rollback_of_version !== null ? m.deploy_rolled_back({ number: one.rollback_of_version }) : one.version === first && one.status === 'succeeded' ? m.deploy_first() : m.deploy_number({ number: one.version });
		const state: DeployRowData['state'] = one.is_live ? 'live' : one.status === 'running' ? 'running' : one.status === 'failed' ? 'failed' : one.can_roll_back ? 'rollback' : 'kept';
		return {
			id: one.id,
			number: `#${one.version}`,
			title,
			meta: [when(one.created_at, now, timeZone), ...(who ? [m.deploy_by({ name: who })] : [])].join(' · '),
			state,
			...(one.error && { error: one.error }),
			version: one.version,
		};
	});
}
