import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { dsApi } from './plugin';

// The docs read every component's API at build time, from the real sources.
describe('dsApi', () => {
	it("serves virtual:ds-api with each component's props, notes and relations", async () => {
		const plugin = dsApi();
		const root = fileURLToPath(new URL('../../../..', import.meta.url));
		(plugin.configResolved as (config: { root: string }) => void)({ root });
		const id = (plugin.resolveId as (source: string) => string)('virtual:ds-api');
		expect(id).toBe('\0virtual:ds-api');
		const watched: string[] = [];
		const code = (plugin.load as (this: unknown, id: string) => string).call({ addWatchFile: (file: string) => watched.push(file) }, id);
		const api = JSON.parse(code.replace(/^export default /, '').replace(/;\n?$/, ''));

		expect(api['actions/Button.svelte'].props.find((prop: { name: string }) => prop.name === 'variant').default).toBe("'primary'");
		expect(api['actions/Button.svelte'].sections).toContainEqual({ sheet: 'nomi', title: 'Actions', id: 'actions' });
		expect(api['foundations/Icon.svelte'].usedBy).toContain('actions/Button.svelte');
		expect(Object.keys(api).some((file) => file.endsWith('Fixture.svelte'))).toBe(false);
		expect(Object.keys(api).length).toBeGreaterThan(150);
		expect(watched.some((file) => file.endsWith('ds/actions/Button.svelte'))).toBe(true);
	});
});
