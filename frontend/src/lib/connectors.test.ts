import { describe, expect, it } from 'vitest';
import { CONNECTORS, connectorState, searchConnectors, serviceAccessLine, withCatalog } from './connectors';
import type { CatalogApp, ConnectionsOverview } from '$lib/types';

describe('searchConnectors', () => {
	it('finds connectors by name, what they do and their category', () => {
		expect(searchConnectors(CONNECTORS, 'hevy').map((c) => c.id)).toEqual(['hevy']);
		expect(searchConnectors(CONNECTORS, 'GMAIL').map((c) => c.id)).toEqual(['google']);
		expect(searchConnectors(CONNECTORS, 'health').map((c) => c.id)).toEqual(['hevy', 'samsung_health', 'apple_health']);
		expect(searchConnectors(CONNECTORS, '  ')).toHaveLength(CONNECTORS.length);
		expect(searchConnectors(CONNECTORS, 'nothing like this')).toEqual([]);
	});
});

const app = (over: Partial<CatalogApp>): CatalogApp => ({
	id: 'notion',
	name: 'Notion',
	category: 'work',
	for_agent: 'workspace',
	access: ['read', 'write'],
	available: false,
	...over,
});

describe('withCatalog', () => {
	it('adds the catalog apps not listed yet, coming soon unless they connect today', () => {
		const all = withCatalog(CONNECTORS, [
			app({ id: 'google_workspace', name: 'Google Workspace', available: true }),
			app({}),
			app({ id: 'github', name: 'GitHub', category: 'building', for_agent: 'coding', access: ['write'], available: true }),
		]);
		expect(all.filter((c) => c.name === 'Google Workspace')).toHaveLength(1);
		const notion = all.find((c) => c.id === 'notion');
		expect(notion?.soon).toBe(true);
		expect(notion?.category).toBe('productivity');
		const github = all.find((c) => c.id === 'github');
		expect(github?.soon).toBe(false);
		expect(github?.elsewhere).toBe(true);
		expect(github?.category).toBe('building');
	});
});

describe('connectorState', () => {
	const google = CONNECTORS.find((c) => c.id === 'google')!;
	const hevy = CONNECTORS.find((c) => c.id === 'hevy')!;
	const overview = (over: Partial<ConnectionsOverview['google']>, hevyOver: Partial<NonNullable<ConnectionsOverview['hevy']>> | null = null): ConnectionsOverview => ({
		google: { configured: true, email: 'ana@example.com', services: ['gmail'], ...over },
		hevy: hevyOver ? { provider: 'hevy', account_name: 'ana', connected_at: '2026-11-01T00:00:00Z', last_used_at: null, ...hevyOver } : null,
	});

	it('shows each Google service with its access, and who uses it', () => {
		const state = connectorState(google, overview({ status: 'ok', used_by: ['workspace'], service_access: [{ service: 'gmail', access: ['read', 'write', 'ask'] }] }));
		expect(state.connected).toBe(true);
		expect(state.needsYou).toBe(false);
		expect(state.access).toEqual(['Gmail · read · write · asks first']);
		expect(state.usedBy).toEqual(['workspace']);
	});

	it('asks the person to reconnect a lapsed sign-in', () => {
		const state = connectorState(google, overview({ status: 'expired', expired_at: '2026-11-10T00:00:00Z' }));
		expect(state).toMatchObject({ connected: false, needsYou: true });
		expect(connectorState(hevy, overview({}, { status: 'expired' }))).toMatchObject({ needsYou: true });
		expect(connectorState(hevy, overview({}, { status: 'ok' })).text).toContain('ana');
	});

	it('says where GitHub is connected from, and whether it lapsed', () => {
		const github = withCatalog([], [app({ id: 'github', name: 'GitHub', available: true })])[0];
		expect(connectorState(github, null, { oauth: false, connection: null }).connected).toBe(false);
		const lapsed = { login: 'ana', kind: 'token' as const, created_at: '', last_used_at: null, status: 'expired' as const };
		expect(connectorState(github, null, { oauth: false, connection: lapsed }).needsYou).toBe(true);
	});

	it('reads the access line', () => {
		expect(serviceAccessLine({ service: 'drive', access: ['read'] })).toBe('Drive · read');
	});
});
