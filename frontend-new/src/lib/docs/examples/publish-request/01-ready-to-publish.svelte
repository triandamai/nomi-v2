<!-- Ready to publish: a teammate's change, built and tested in the preview. -->
<script lang="ts">
	import PublishRequest from '$lib/ds/build/PublishRequest.svelte';
</script>

<div class="w-full max-w-140">
	<PublishRequest
		who={{ name: 'Sari Wulandari', initials: 'SW', peer: 1 }}
		meta="Makanan section · 2 files · built and tested in the preview"
		onpublish={() => {}}
		onreview={() => {}}
	/>
</div>
