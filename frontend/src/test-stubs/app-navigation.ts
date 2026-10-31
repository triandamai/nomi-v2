// Test-only stand-in for SvelteKit's virtual `$app/navigation` module. Component tests that need to
// see a navigation spy on it with `vi.mock('$app/navigation', ...)`.
export const goto = async () => {};
export const invalidateAll = async () => {};
export const afterNavigate = () => {};
export const beforeNavigate = () => {};
