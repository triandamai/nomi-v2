<!-- Before it runs: the steps, what you'll have, one clear choice, and the cost underneath. -->
<script lang="ts">
	import BuildPlan from '$lib/ds/build/BuildPlan.svelte';
</script>

<div class="w-full max-w-140">
	<BuildPlan
		project="warung-kopi-menu"
		summary="Menu and pre-order page for Warung Kopi Bu Siti"
		steps={[
			'Set up the project and a small database for orders',
			"Menu page with Bu Siti's items, prices and photos",
			'Pre-order form with 15-minute pickup slots',
			"Orders view so Bu Siti sees what's coming",
			'Test it on a phone',
			'Put it online at a link you can share',
		]}
		outcome="When it's done: a live link customers can open today. The code is yours to export."
		options={[
			{ title: 'Build it', detail: "Koda builds it and puts it online. You see each file as it's written and can stop any time.", recommended: true },
			{ title: 'Build it, ask before it goes live', detail: 'Everything is built in the preview first. Customers see it only after you press Publish.' },
		]}
		cost="About 5 min · included in your free plan"
		onrun={() => {}}
	/>
</div>
