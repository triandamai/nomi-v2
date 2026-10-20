// Test-only stand-in for SvelteKit's virtual `$app/environment` module (component tests run in jsdom).
export const browser = true;
export const dev = true;
export const building = false;
export const version = 'test';
