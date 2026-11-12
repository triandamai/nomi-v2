<!-- Default: the language in use; the platform's own picker opens from it. -->
<script lang="ts">
	import LanguageSwitch from '$lib/ds/auth/LanguageSwitch.svelte';

	let language = $state('en');
</script>

<LanguageSwitch
	value={language}
	options={[
		{ value: 'en', label: 'English' },
		{ value: 'id', label: 'Bahasa Indonesia' },
	]}
	onchange={(value) => (language = value)}
/>
