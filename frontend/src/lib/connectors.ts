// The apps people can connect on Connections, and what each one lets the crew do. Each has its
// own setup page (/connections/<id>); the ones marked `soon` are listed but not open yet.
import { m } from '$lib/paraglide/messages';
import { timeAgo } from '$lib/i18n';
import type { CatalogApp, ConnectionAccess, ConnectionsOverview, GithubStatus, ServiceAccess } from '$lib/types';

/** One of the apps below, or one from the server's catalog (GET /api/connections/catalog). */
export type ConnectorId = string;
export type ConnectorCategory = 'productivity' | 'building' | 'health' | 'money' | 'home_media';

export interface Connector {
	id: ConnectorId;
	name: string;
	category: ConnectorCategory;
	/** The crew member who works with it (an agent_type). */
	agent: string;
	description: () => string;
	/** An icon path (24×24, stroked). */
	icon: string;
	soon?: boolean;
	/** Connected somewhere else (GitHub, from a project's Export): listed without a setup page. */
	elsewhere?: boolean;
}

export const CONNECTORS: Connector[] = [
	{
		id: 'google',
		name: 'Google Workspace',
		category: 'productivity',
		agent: 'workspace',
		description: m.connectors_google_desc,
		icon: 'M4 6h16v12H4z M4 7l8 6 8-6',
	},
	{
		id: 'hevy',
		name: 'Hevy',
		category: 'health',
		agent: 'health',
		description: m.connectors_hevy_desc,
		icon: 'M6 8v8 M18 8v8 M3 10v4 M21 10v4 M6 12h12',
	},
	{
		id: 'samsung_health',
		name: 'Samsung Health',
		category: 'health',
		agent: 'health',
		description: m.connectors_samsung_desc,
		icon: 'M20.8 6.6a5 5 0 0 0-8.8-2.2 5 5 0 0 0-8.8 2.2c-.7 3.9 3.2 7.3 8.8 12.4 5.6-5.1 9.5-8.5 8.8-12.4z',
		soon: true,
	},
	{
		id: 'apple_health',
		name: 'Apple Health',
		category: 'health',
		agent: 'health',
		description: m.connectors_apple_desc,
		icon: 'M20.8 6.6a5 5 0 0 0-8.8-2.2 5 5 0 0 0-8.8 2.2c-.7 3.9 3.2 7.3 8.8 12.4 5.6-5.1 9.5-8.5 8.8-12.4z',
		soon: true,
	},
];

export const CATEGORY_ORDER: ConnectorCategory[] = ['productivity', 'building', 'health', 'money', 'home_media'];

export function categoryLabel(category: ConnectorCategory): string {
	switch (category) {
		case 'health':
			return m.connectors_cat_health();
		case 'building':
			return m.connectors_cat_building();
		case 'money':
			return m.connectors_cat_money();
		case 'home_media':
			return m.connectors_cat_home_media();
		default:
			return m.connectors_cat_productivity();
	}
}

/** The catalog's ids for the apps listed above. */
const CATALOG_IDS: Record<string, ConnectorId> = { google_workspace: 'google', hevy: 'hevy' };
const CATALOG_CATEGORY: Record<CatalogApp['category'], ConnectorCategory> = {
	work: 'productivity',
	building: 'building',
	health: 'health',
	money: 'money',
	home_media: 'home_media',
};
/** A plain plug, for apps without an icon of their own. */
const PLUG_ICON = 'M9 2v6 M15 2v6 M6 8h12v4a6 6 0 0 1-12 0z M12 18v4';

export function accessLabel(access: ConnectionAccess): string {
	if (access === 'read') return m.connectors_access_read();
	if (access === 'write') return m.connectors_access_write();
	return m.connectors_access_ask();
}

/** "Gmail · read · write · asks first". */
export function serviceAccessLine({ service, access }: ServiceAccess): string {
	const name = service.charAt(0).toUpperCase() + service.slice(1);
	return [name, ...access.map(accessLabel)].join(' · ');
}

/** The listed apps plus the server's catalog: apps it can't connect yet show as coming soon. */
export function withCatalog(connectors: Connector[], catalog: CatalogApp[]): Connector[] {
	const known = new Set(connectors.map((c) => c.id));
	const more = catalog
		.filter((app) => !known.has(CATALOG_IDS[app.id] ?? app.id))
		.map(
			(app): Connector => ({
				id: app.id,
				name: app.name,
				category: CATALOG_CATEGORY[app.category] ?? 'productivity',
				agent: app.for_agent,
				description:
					app.id === 'github' ? m.connectors_github_desc : () => m.connectors_access({ access: app.access.map(accessLabel).join(', ') }),
				icon: PLUG_ICON,
				soon: !app.available,
				elsewhere: app.available && app.id === 'github',
			}),
		);
	return [...connectors, ...more];
}

/** What a row says about this person's connection: connected, lapsed (needs them), or not. */
export interface ConnectorState {
	connected: boolean;
	needsYou: boolean;
	text: string;
	/** The crew members that use it (agent types). */
	usedBy: string[];
	/** Per-service access lines (Google). */
	access: string[];
}

export function connectorState(connector: Connector, overview: ConnectionsOverview | null, github: GithubStatus | null = null): ConnectorState {
	const base: ConnectorState = { connected: false, needsYou: false, text: m.conn_not_connected(), usedBy: [connector.agent], access: [] };
	if (connector.soon) return { ...base, text: m.connectors_soon() };
	const lastUsed = (when: string | null | undefined) => (when ? ` · ${m.connectors_last_used({ when: timeAgo(when) })}` : '');
	if (connector.id === 'google' && overview?.google.email) {
		const google = overview.google;
		const expired = google.status === 'expired';
		return {
			connected: !expired,
			needsYou: expired,
			text: expired ? m.connectors_needs_you() : google.email + lastUsed(google.last_used_at),
			usedBy: google.used_by?.length ? google.used_by : base.usedBy,
			access: (google.service_access ?? []).map(serviceAccessLine),
		};
	}
	if (connector.id === 'hevy' && overview?.hevy) {
		const hevy = overview.hevy;
		const expired = hevy.status === 'expired';
		const who = hevy.account_name ?? m.connectors_connected_since({ when: timeAgo(hevy.connected_at) });
		return {
			...base,
			connected: !expired,
			needsYou: expired,
			text: expired ? m.connectors_needs_you() : who + lastUsed(hevy.last_used_at),
			usedBy: hevy.used_by?.length ? hevy.used_by : base.usedBy,
		};
	}
	if (connector.id === 'github') {
		const connection = github?.connection;
		if (!connection) return { ...base, text: m.connectors_github_from_export(), usedBy: [] };
		const expired = connection.status === 'expired';
		return {
			...base,
			connected: !expired,
			needsYou: expired,
			text: expired ? m.connectors_needs_you() : connection.login + lastUsed(connection.last_used_at),
			usedBy: [],
		};
	}
	return base;
}

/** Connectors whose name, description or category has every word of `query`. */
export function searchConnectors(connectors: Connector[], query: string): Connector[] {
	const words = query.toLowerCase().split(/\s+/).filter(Boolean);
	if (words.length === 0) return connectors;
	return connectors.filter((c) => {
		const text = `${c.name} ${c.description()} ${categoryLabel(c.category)}`.toLowerCase();
		return words.every((w) => text.includes(w));
	});
}
