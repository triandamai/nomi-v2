<script lang="ts">
	import BuildPlan from '$lib/ds/build/BuildPlan.svelte';
	import BuildRunning from '$lib/ds/build/BuildRunning.svelte';
	import BuildDone from '$lib/ds/build/BuildDone.svelte';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';

	const noop = () => {};
</script>

<GallerySection
	title="Build estimate"
	description="Shown before anything runs, like a Claude plan: the steps, what you'll have, and one clear choice. Cost is a quiet line, never the headline. It collapses into the thread once the build starts."
>
	<div class="grid items-start gap-6 lg:grid-cols-2">
		<GalleryBoard title="Build estimate" note="default · before run" inline>
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
				onrun={noop}
			/>
		</GalleryBoard>

		<GalleryBoard title="Running and done" note="collapsed while building · actuals after deploy" inline>
			<div class="flex flex-col gap-5">
				<BuildRunning started="08:13" step="writing src/routes/menu/+page.svelte" elapsed="01:12" current={2} total={6} label="menu page" eta="~3 min left" onstop={noop} />
				<BuildDone meta="deploy #1 · 08:18" url="https://warung-kopi-menu-3f9a12.trian.space" summary="Built in 5m 40s · used your free allowance" ondetails={noop} onshare={noop} />
			</div>
		</GalleryBoard>
	</div>
</GallerySection>
