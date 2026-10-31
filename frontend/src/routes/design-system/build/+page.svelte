<script lang="ts">
	import SheetHeader from '$lib/gallery/SheetHeader.svelte';
	import EstimateSection from '$lib/gallery/build/EstimateSection.svelte';
	import BuildingSection from '$lib/gallery/build/BuildingSection.svelte';
	import ShareSection from '$lib/gallery/build/ShareSection.svelte';
	import FirstRunSection from '$lib/gallery/build/FirstRunSection.svelte';
	import WorkspaceSection from '$lib/gallery/build/WorkspaceSection.svelte';
	import CodeSection from '$lib/gallery/build/CodeSection.svelte';
	import TogetherSection from '$lib/gallery/build/TogetherSection.svelte';
	import MobileBuildSection from '$lib/gallery/build/MobileBuildSection.svelte';
	import StorySection from '$lib/gallery/build/StorySection.svelte';
	import TypesSection from '$lib/gallery/build/TypesSection.svelte';

	// frontend-new's Build sheet: its sections as they are, the twins drawn in M3 through ds-twins.css.
</script>

<main class="mx-auto flex max-w-400 flex-col gap-16 px-4 py-12 md:px-16">
	<SheetHeader
		eyebrow="DESIGN SYSTEM · BUILD"
		title="Everything between describe and a live URL"
		lede="Koda shows the plan and a cost range before anything runs, then the real step it is on, never a progress bar. Every number here comes from one story: warung-kopi-menu, started 08:13."
	/>

	<EstimateSection />
	<BuildingSection />
	<ShareSection />
	<FirstRunSection />
	<WorkspaceSection />
	<CodeSection />
	<TogetherSection />
	<MobileBuildSection />
	<StorySection />
	<TypesSection />
</main>
