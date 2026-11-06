// Test-only stand-in for SvelteKit's virtual `$app/forms` module. Actions answer with devalue in the
// app; the tests answer with plain JSON, which is all `deserialize` needs to read here.
export function deserialize(text: string) {
	return JSON.parse(text);
}

// `use:enhance` does nothing in jsdom: the form stays a plain form, and tests drive submits through
// the page's own handlers.
export function enhance(_form: HTMLFormElement, _submit?: unknown) {
	return { destroy() {} };
}

export async function applyAction(_result: unknown) {}
