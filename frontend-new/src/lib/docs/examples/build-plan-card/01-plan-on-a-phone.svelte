<!-- Plan on a phone: the steps, what you get, one big Build it, the cost under it. -->
<script lang="ts">
	import BuildPlanCard from '$lib/ds/mobile/BuildPlanCard.svelte';
</script>

<div class="w-89.5 max-w-full">
	<BuildPlanCard
		project="warung-kopi-menu"
		summary="Menu and pre-order page for Warung Kopi Bu Siti"
		steps={[
			'Set up the project and a small database',
			'Menu page with items, prices and photos',
			'Pre-order form with 15-minute pickup slots',
			'Orders view for Bu Siti',
			'Test it on a phone',
			'Put it online at a link you can share',
		]}
		outcome="A live link customers can open today. The code is yours."
		cost="About 5 min · included in your free plan"
		onbuild={() => {}}
		onaskfirst={() => {}}
		onkeep={() => {}}
	/>
</div>
