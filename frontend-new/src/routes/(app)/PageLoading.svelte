<script lang="ts">
	// While another page loads (its server load runs before anything changes), this stands in its
	// place: a title block and rows, shimmering, with a line saying which page is on its way. It
	// keeps the page's own padding and never runs wider than the screen.
	let { label }: { label: string } = $props();
	const ROWS = 5;
</script>

<div role="status" aria-busy="true" aria-live="polite" class="flex w-full min-w-0 flex-col gap-4 px-4 py-6 lg:px-14 lg:py-8" data-page-loading>
	<div class="flex flex-col gap-2" aria-hidden="true">
		<span class="h-7 w-48 max-w-full rounded-sm shimmer"></span>
		<span class="h-3 w-72 max-w-full rounded-2xs shimmer"></span>
	</div>
	<div class="flex flex-col gap-2.5" aria-hidden="true">
		{#each { length: ROWS } as _, row (row)}
			<div class="flex items-center gap-3 rounded-md p-4 outline -outline-offset-1 outline-border">
				<span class="size-8 shrink-0 rounded-sm shimmer"></span>
				<span class="flex min-w-0 flex-1 flex-col gap-1.5">
					<span class="h-2.5 w-35 max-w-full rounded-2xs shimmer"></span>
					<span class="h-2 w-full rounded-2xs shimmer"></span>
				</span>
			</div>
		{/each}
	</div>
	<span class="font-mono text-label text-text-3">{label}</span>
</div>
