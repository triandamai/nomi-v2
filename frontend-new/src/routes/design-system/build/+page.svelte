<script lang="ts">
	import EstimateSection from '$lib/gallery/build/EstimateSection.svelte';
	import BuildingSection from '$lib/gallery/build/BuildingSection.svelte';
	import ShareSection from '$lib/gallery/build/ShareSection.svelte';
	import FirstRunSection from '$lib/gallery/build/FirstRunSection.svelte';
	import WorkspaceSection from '$lib/gallery/build/WorkspaceSection.svelte';
	import CodeSection from '$lib/gallery/build/CodeSection.svelte';
	import TogetherSection from '$lib/gallery/build/TogetherSection.svelte';
</script>

<header class="flex max-w-xl flex-col gap-3 py-12">
	<p class="font-mono text-label tracking-label text-text-3 uppercase">Design system · Build</p>
	<h1 class="text-display font-medium tracking-display text-text">Everything between describe and a live URL</h1>
	<p class="text-body text-text-2">
		Koda shows the plan and a cost range before anything runs, then the real step it is on, never a progress bar. Every number here comes from one
		story: warung-kopi-menu, started 08:13.
	</p>
</header>

<EstimateSection />
<BuildingSection />
<ShareSection />
<FirstRunSection />
<WorkspaceSection />
<CodeSection />
<TogetherSection />
