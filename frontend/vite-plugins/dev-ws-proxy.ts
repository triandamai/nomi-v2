import type { Plugin } from 'vite';
import { attachWsProxy } from '../ws-proxy/ws-proxy.js';

/**
 * Wires the same relay used in production (server.js) into `vite dev` and `vite preview` — the
 * latter is what the Playwright e2e harness boots via `npm run build && npm run preview`,
 * since SvelteKit's own Vite plugin serves the SSR build through `vite preview` via
 * `configurePreviewServer`, independently of adapter-node's server.js.
 */
export function devWsProxy(): Plugin {
	return {
		name: 'dev-ws-proxy',
		configureServer(server) {
			attachWsProxy(server.httpServer);
		},
		configurePreviewServer(server) {
			attachWsProxy(server.httpServer);
		}
	};
}
