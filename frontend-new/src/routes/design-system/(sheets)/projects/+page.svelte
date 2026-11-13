<script lang="ts">
	import SheetHeader from '$lib/gallery/SheetHeader.svelte';
	import PagePartsSection from '$lib/gallery/projects/PagePartsSection.svelte';
	import CardsSection from '$lib/gallery/projects/CardsSection.svelte';
</script>

<SheetHeader
	eyebrow="DESIGN SYSTEM · PROJECTS"
	title="Every project, its state and its next action"
	lede="The Projects page and its flows are built only from these. Cards have a fixed size and sit in a left-aligned grid: two per row beside the panel, three when it's closed, and empty space when there are fewer."
/>

<PagePartsSection />
<CardsSection />
