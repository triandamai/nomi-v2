<!-- Three versions of how Nomi speaks. -->
<script lang="ts">
	import PersonalityVersions from '$lib/ds/settings/PersonalityVersions.svelte';
</script>

<div class="w-full max-w-212">
	<PersonalityVersions
		title="personality"
		count="v3 · 3 versions"
		labels={{ current: 'current', restore: 'Restore' }}
		versions={[
			{ version: 3, description: 'Warm and brief, a little dry humour', date: '12 Oct', current: true },
			{ version: 2, description: 'Formal, full sentences', date: '2 Oct', current: false },
			{ version: 1, description: 'Friendly and chatty', date: '20 Sep', current: false },
		]}
	/>
</div>
