// Test-only stand-in for SvelteKit's virtual `$app/forms` module. Actions answer with devalue in the
// app; the tests answer with plain JSON, which is all `deserialize` needs to read here.
export function deserialize(text: string) {
	return JSON.parse(text);
}
