// Test-only stand-in for SvelteKit's virtual `$app/state` module, which plain Vitest cannot resolve.
// Component tests override it with `vi.mock('$app/state', ...)` when they need a specific URL.
export const page = { url: new URL('http://localhost/'), params: {}, data: {} };
export const navigating = { to: null };
export const updated = { current: false };
