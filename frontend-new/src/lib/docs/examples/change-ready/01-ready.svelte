<!-- Ready: a teammate's change, to publish or review first. -->
<script lang="ts">
	import ChangeReady from '$lib/ds/mobile/ChangeReady.svelte';
</script>

<div class="w-89.5 max-w-full">
	<ChangeReady who={{ name: 'Sari', initials: 'SW', peer: 1 }} meta="Makanan section · 2 files · tests pass" onpublish={() => {}} onreview={() => {}} />
</div>
