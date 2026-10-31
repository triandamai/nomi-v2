<script lang="ts">
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import SettingsNav from '$lib/components/ds/chrome/SettingsNav.svelte';
	import { activeSetting, filterSettings, settingsGroups } from '$lib/settingsNav';
	import { VERSION_LABEL } from '$lib/version';
	import type { Profile } from '$lib/types';

	// The account pages' drawer, in place of the main navigation: back to Nomi (or Esc), a search
	// over the settings, the groups, and the person with Log out and the app's version.
	let { userEmail, profile }: { userEmail: string; profile: Profile } = $props();

	let query = $state('');
	let logout: HTMLFormElement | undefined = $state();

	const groups = $derived(filterSettings(settingsGroups(), query));

	function isEditable(target: EventTarget | null): boolean {
		return target instanceof HTMLElement && (target.isContentEditable || ['INPUT', 'TEXTAREA', 'SELECT'].includes(target.tagName));
	}

	// Esc goes back to Nomi, as the drawer says, unless it's closing something on the page or
	// leaving a field.
	$effect(() => {
		const onkey = (event: KeyboardEvent) => {
			if (event.key !== 'Escape' || event.defaultPrevented || isEditable(event.target) || document.querySelector('dialog[open]')) return;
			goto('/');
		};
		window.addEventListener('keydown', onkey);
		return () => window.removeEventListener('keydown', onkey);
	});
</script>

<SettingsNav
	backHref="/"
	{groups}
	active={activeSetting(page.url.pathname)}
	user={{ name: profile.display_name || userEmail, detail: profile.display_name ? userEmail : undefined, avatar: profile.avatar_url ?? undefined }}
	version={VERSION_LABEL}
	bind:query
	onlogout={() => logout?.requestSubmit()}
/>
<form bind:this={logout} method="POST" action="/logout" hidden></form>
