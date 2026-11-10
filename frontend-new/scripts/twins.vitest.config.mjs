import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';

// The old app's twin-parity test (frontend/src/lib/dsParity.test.ts) keeps its copies of this
// design system identical to ours. It only reads files, so it runs here on this app's Vitest,
// without the old app's dependencies: a change to src/lib/ds that would break it fails our CI.
const frontend = fileURLToPath(new URL('../../frontend/', import.meta.url));
const vitest = fileURLToPath(new URL('../node_modules/vitest/dist/index.js', import.meta.url));

export default defineConfig({
	root: frontend,
	resolve: { alias: { vitest } },
	server: { fs: { allow: [frontend, fileURLToPath(new URL('..', import.meta.url))] } },
	test: { include: ['src/lib/dsParity.test.ts'], environment: 'node' },
});
