<script lang="ts">
	import { page } from '$app/state';
	import AuthLayout from '$lib/ds/auth/AuthLayout.svelte';
	import LanguageSwitch from '$lib/ds/auth/LanguageSwitch.svelte';
	import { m } from '$lib/paraglide/messages';
	import { getLocale, isLocale, setLocale } from '$lib/paraglide/runtime';

	let { data, children } = $props();

	// Before sign-in the language is chosen here: Paraglide keeps it in its cookie and reloads.
	const languages = [
		{ value: 'en', label: m.language_en() },
		{ value: 'id', label: m.language_id() },
	];
</script>

<AuthLayout version={data.version} year={data.year} healthy={data.healthy} greeting={page.data.greeting === false ? false : undefined}>
	{#snippet top()}
		<LanguageSwitch value={getLocale()} options={languages} onchange={(value) => isLocale(value) && setLocale(value)} />
	{/snippet}
	{@render children()}
</AuthLayout>
