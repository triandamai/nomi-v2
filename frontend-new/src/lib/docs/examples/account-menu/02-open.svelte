<!-- Open, as the pen draws it: inline shows the panel in place. -->
<script lang="ts">
	import AccountMenu from '$lib/ds/chrome/AccountMenu.svelte';
</script>

<AccountMenu
	inline
	name="Mario Prasetyo"
	plan="Free · 21.8K / 1M tokens"
	upgradeHref="#plan"
	settingsHref="#settings"
	profileHref="#profile"
	planHref="#plan"
	theme="dark"
	ontheme={() => {}}
	language="en"
	languages={[
		{ value: 'en', label: 'English' },
		{ value: 'id', label: 'Bahasa Indonesia' },
	]}
	onlanguage={() => {}}
	version="v0.0.75"
	onshortcuts={() => {}}
	onlogout={() => {}}
/>
