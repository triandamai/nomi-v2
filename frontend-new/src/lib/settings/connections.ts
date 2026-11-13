import { m } from '$lib/paraglide/messages';
import { getLocale } from '$lib/paraglide/runtime';
import type { ConnectionsOverview, GithubStatus } from '$lib/adapters/api';
import type { CrewId } from '$lib/ds/agents/crew';

// Settings › Connections (App / Settings / Connections OBmgS): the apps the crew can work in, as
// the backend connects them: Google Workspace (the services the person grants), GitHub (signed in,
// or a token) and Hevy (an API key). Each is connected (what it can do, who uses it) or one to add.

export type AppId = 'google' | 'github' | 'hevy';
export const GOOGLE_SERVICES = ['gmail', 'docs', 'sheets', 'drive', 'calendar'] as const;
export type GoogleService = (typeof GOOGLE_SERVICES)[number];

export type ConnectedApp = { id: AppId; account: string; caption: string; can: string[]; usedBy: { agent: CrewId; name: string }[]; lastUsed: string | null };
export type AppToAdd = { id: AppId; caption: string; for: { agent: CrewId; name: string }; access: string; available: boolean };

const SERVICE_NAMES: Record<string, () => string> = {
	gmail: m.connections_gmail,
	docs: m.connections_docs,
	sheets: m.connections_sheets,
	drive: m.connections_drive,
	calendar: m.connections_calendar,
};
export const serviceName = (service: string) => (SERVICE_NAMES[service] ?? (() => service))();

/** "ko••@gmail.com": the first two letters, then the domain. */
export const maskAddress = (email: string) => {
	const [local, domain] = email.split('@');
	return `${local.slice(0, 2)}••@${domain ?? ''}`;
};

export function connectedApps(overview: ConnectionsOverview | null, github: GithubStatus | null): ConnectedApp[] {
	const apps: ConnectedApp[] = [];
	if (overview?.google.email && overview.google.services.length) {
		const services = GOOGLE_SERVICES.filter((service) => overview.google.services.includes(service));
		apps.push({
			id: 'google',
			account: maskAddress(overview.google.email),
			caption: new Intl.ListFormat(getLocale() === 'id' ? 'id' : 'en', { type: 'conjunction' }).format(services.map(serviceName)),
			can: services.map(serviceName),
			usedBy: [{ agent: 'tara', name: 'Tara' }],
			lastUsed: null,
		});
	}
	if (github?.connection) {
		apps.push({
			id: 'github',
			account: `@${github.connection.login}`,
			caption: github.connection.kind === 'token' ? m.connections_github_token() : m.connections_github_oauth(),
			can: [m.connections_can_import(), m.connections_can_push()],
			usedBy: [{ agent: 'koda', name: 'Koda' }],
			lastUsed: github.connection.last_used_at,
		});
	}
	if (overview?.hevy) {
		apps.push({
			id: 'hevy',
			account: overview.hevy.account_name ?? m.connections_hevy_account(),
			caption: m.connections_hevy_caption(),
			can: [m.connections_can_workouts()],
			usedBy: [{ agent: 'raga', name: 'Raga' }],
			lastUsed: overview.hevy.last_used_at,
		});
	}
	return apps;
}

/** The apps not connected yet; Google only where whoever runs this Nomi set it up. */
export function appsToAdd(overview: ConnectionsOverview | null, github: GithubStatus | null): AppToAdd[] {
	const connected = new Set(connectedApps(overview, github).map((app) => app.id));
	const all: AppToAdd[] = [
		{ id: 'google', caption: m.connections_google_add(), for: { agent: 'tara', name: 'Tara' }, access: m.connections_access_read_write(), available: !!overview?.google.configured },
		{ id: 'github', caption: m.connections_github_add(), for: { agent: 'koda', name: 'Koda' }, access: m.connections_access_read_write(), available: github !== null },
		{ id: 'hevy', caption: m.connections_hevy_add(), for: { agent: 'raga', name: 'Raga' }, access: m.connections_access_read(), available: overview !== null },
	];
	return all.filter((app) => !connected.has(app.id));
}
