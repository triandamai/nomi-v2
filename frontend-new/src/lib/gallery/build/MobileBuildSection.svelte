<script lang="ts">
	import ProjectNav from '$lib/ds/mobile/ProjectNav.svelte';
	import WorkspaceSwitch from '$lib/ds/mobile/WorkspaceSwitch.svelte';
	import AskBar from '$lib/ds/mobile/AskBar.svelte';
	import BuildPlanCard from '$lib/ds/mobile/BuildPlanCard.svelte';
	import BuildProgress from '$lib/ds/mobile/BuildProgress.svelte';
	import LiveCard from '$lib/ds/mobile/LiveCard.svelte';
	import DnsRecord from '$lib/ds/mobile/DnsRecord.svelte';
	import TeammateBubble from '$lib/ds/mobile/TeammateBubble.svelte';
	import AskQueue from '$lib/ds/mobile/AskQueue.svelte';
	import ChangeReady from '$lib/ds/mobile/ChangeReady.svelte';
	import GallerySection from '../GallerySection.svelte';
	import GalleryBoard from '../GalleryBoard.svelte';
	import WarungApp from './WarungApp.svelte';

	const noop = () => {};
	const sari = { name: 'Sari', initials: 'SW', peer: 1 as const };
	const steps = [
		'Set up the project and a small database',
		'Menu page with items, prices and photos',
		'Pre-order form with 15-minute pickup slots',
		'Orders view for Bu Siti',
		'Test it on a phone',
		'Put it online at a link you can share',
	];
	const asks = [
		{ text: 'Make the Pre-order button green', who: { name: 'Mario', initials: 'MP' }, state: 'running' as const, meta: 'Mario · running · 00:21' },
		{ text: 'Add a Makanan section with nasi uduk', who: sari, state: 'next' as const, meta: 'Sari · next' },
	];
</script>

<GallerySection
	title="Build on mobile"
	description="One column at 390 pt. Chat and Preview switch with a segmented control; every project tool is a bottom sheet; the live app gets the full screen with Koda one tap away."
>
	<div class="flex flex-col gap-6">
		<div class="flex flex-wrap items-start gap-8">
			<GalleryBoard title="Project bar" note="nav + Chat | Preview switch" inline phone>
				<div class="flex w-97.5 flex-col gap-5">
					<ProjectNav project="warung-kopi-menu" status="live · deploy #2" publish="idle" onback={noop} onmenu={noop} />
					<WorkspaceSwitch />
				</div>
			</GalleryBoard>
			<GalleryBoard title="Ask bar" note="floats over the live preview" inline phone>
				<div class="w-89.5 max-w-full"><AskBar onask={noop} onvoice={noop} /></div>
			</GalleryBoard>
			<GalleryBoard title="Live app" note="the built app at phone size, light" inline phone>
				<div data-theme="light" class="flex h-150 w-97.5 flex-col overflow-hidden bg-bg text-text"><WarungApp /></div>
			</GalleryBoard>
		</div>
		<div class="flex flex-wrap items-start gap-8">
			<GalleryBoard title="Plan on a phone" note="steps, outcome, big run choices" inline phone>
				<div class="w-89.5 max-w-full">
					<BuildPlanCard
						project="warung-kopi-menu"
						summary="Menu and pre-order page for Warung Kopi Bu Siti"
						{steps}
						outcome="A live link customers can open today. The code is yours."
						cost="About 5 min · included in your free plan"
						onbuild={noop}
						onaskfirst={noop}
						onkeep={noop}
					/>
				</div>
			</GalleryBoard>
			<GalleryBoard title="Building and live" note="one card, real step, notify when done" inline phone>
				<div class="flex w-89.5 max-w-full flex-col gap-5">
					<BuildProgress project="warung-kopi-menu" elapsed="01:12" step="writing src/routes/menu/+page.svelte" current={2} total={6} label="menu page" eta="~3 min left" onstop={noop} />
					<LiveCard meta="deploy #1 · 08:18" url="https://warung-kopi-menu-3f9a12.trian.space" summary="Built in 5m 40s · included in your free plan" onshare={noop} />
				</div>
			</GalleryBoard>
		</div>
		<div class="flex flex-wrap items-start gap-8">
			<GalleryBoard title="Domain record" note="one record per card, copy in one tap" inline phone>
				<div class="w-89.5 max-w-full"><DnsRecord type="A" name="@" value="76.76.21.21" state="waiting" /></div>
			</GalleryBoard>
			<GalleryBoard title="Teammate in chat" note="their colour, their name, who they asked" inline phone>
				<div class="w-89.5 max-w-full">
					<TeammateBubble who={sari} time="09:14" to="Koda" text="Can we add a Makanan section? Nasi uduk and roti bakar, same pickup times." />
				</div>
			</GalleryBoard>
		</div>
		<div class="flex flex-wrap items-start gap-8">
			<GalleryBoard title="Together on a phone" note="queue and publish request, thumb-sized" inline phone>
				<div class="flex w-89.5 max-w-full flex-col gap-5">
					<AskQueue {asks} onfirst={noop} />
					<ChangeReady who={sari} meta="Makanan section · 2 files · tests pass" onpublish={noop} onreview={noop} />
				</div>
			</GalleryBoard>
		</div>
	</div>
</GallerySection>
