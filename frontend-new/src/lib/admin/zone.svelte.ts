import { onMount } from 'svelte';

/**
 * The time zone the console shows times in: the person's saved one, else the browser's once the
 * page is running (the server renders UTC until then). Call during a component's setup.
 */
export function timeZone(saved: () => string | null | undefined) {
	let browser = $state<string | undefined>();
	onMount(() => {
		browser = Intl.DateTimeFormat().resolvedOptions().timeZone;
	});
	return {
		get value(): string | undefined {
			return saved() ?? browser;
		},
	};
}
