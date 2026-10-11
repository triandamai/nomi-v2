// The apps people can connect on Connections, and what each one lets the crew do. Each has its
// own setup page (/connections/<id>); the ones marked `soon` are listed but not open yet.
import { m } from '$lib/paraglide/messages';

export type ConnectorId = 'google' | 'hevy' | 'samsung_health' | 'apple_health';
export type ConnectorCategory = 'productivity' | 'health';

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

export const CATEGORY_ORDER: ConnectorCategory[] = ['productivity', 'health'];

export function categoryLabel(category: ConnectorCategory): string {
	return category === 'health' ? m.connectors_cat_health() : m.connectors_cat_productivity();
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
