<!-- Before the first deploy: empty and honest, with the plan's real steps. -->
<script lang="ts">
	import PreviewBuilding from '$lib/ds/build/PreviewBuilding.svelte';
</script>

<div class="w-full max-w-140">
	<PreviewBuilding
		steps={[
			{ label: 'Set up project and database', state: 'done' },
			{ label: 'Menu page', state: 'running' },
			{ label: 'Pre-order form', state: 'waiting' },
			{ label: 'Orders view for Bu Siti', state: 'waiting' },
			{ label: 'Test on mobile', state: 'waiting' },
			{ label: 'Deploy to the live link', state: 'waiting' },
		]}
	/>
</div>
