<!-- Describe an app: type to see the accent; once Koda knows what the database is for, its chip lights up. -->
<script lang="ts">
	import ProjectComposer from '$lib/ds/projects/ProjectComposer.svelte';

	let value = $state('');
</script>

<ProjectComposer bind:value domain="trian.space" database={value.toLowerCase().includes('order') ? 'for orders' : undefined} onbuild={() => (value = '')} onimport={() => {}} />
