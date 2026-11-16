<!-- Phone: the New project sheet's composer, Koda knowing what the database is for. -->
<script lang="ts">
	import ProjectComposer from '$lib/ds/projects/ProjectComposer.svelte';

	let value = $state('Order tracker for my kiloan laundry: customers, weight, pickup date, paid or not');
</script>

<div class="w-89.5 max-w-full">
	<ProjectComposer phone bind:value domain="trian.space" database="for orders" onbuild={() => {}} onimport={() => {}} />
</div>
