import type { Plugin } from 'vite';
import { attachSessionStreamProxy } from '../ws-proxy/session-stream-proxy.js';

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
			attachSessionStreamProxy(server.httpServer);
		},
		configurePreviewServer(server) {
			attachSessionStreamProxy(server.httpServer);
		}
	};
}
