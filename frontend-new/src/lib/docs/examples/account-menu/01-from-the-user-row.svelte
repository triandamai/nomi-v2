<!-- From the user row: press the person's row; the theme changes in place, the language opens its own list. -->
<script lang="ts">
	import AccountMenu, { type Theme } from '$lib/ds/chrome/AccountMenu.svelte';

	let theme = $state<Theme>('system');
	let language = $state('en');
</script>

<div class="w-56">
	<AccountMenu
		name="Mario Prasetyo"
		plan="Free · 21.8K / 1M tokens"
		upgradeHref="#plan"
		settingsHref="#settings"
		profileHref="#profile"
		planHref="#plan"
		{theme}
		ontheme={(next) => (theme = next)}
		{language}
		languages={[
			{ value: 'en', label: 'English' },
			{ value: 'id', label: 'Bahasa Indonesia' },
		]}
		onlanguage={(next) => (language = next)}
		version="v0.0.75"
		onshortcuts={() => {}}
		onlogout={() => {}}
	/>
</div>
