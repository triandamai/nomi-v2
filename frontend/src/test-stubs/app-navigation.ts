// Test-only stand-in for SvelteKit's virtual `$app/navigation` module. Component tests that need to
// see a navigation spy on it with `vi.mock('$app/navigation', ...)`.
export const goto = async (..._args: unknown[]) => {};
export const invalidateAll = async () => {};
export const invalidate = async (..._args: unknown[]) => {};
export const afterNavigate = (..._args: unknown[]) => {};
export const beforeNavigate = (..._args: unknown[]) => {};
export const onNavigate = (..._args: unknown[]) => {};
export const replaceState = (..._args: unknown[]) => {};
export const pushState = (..._args: unknown[]) => {};
