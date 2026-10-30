<script lang="ts">
	// Stands in while something loads, and says what ("loading 4 projects…"). Still, not shimmering:
	// honest states, no fake progress.
	let { label }: { label: string } = $props();
</script>

<div role="status" aria-busy="true" class="flex w-full flex-col gap-2.5 rounded-md p-4 outline -outline-offset-1 outline-border">
	<div class="flex items-center gap-3" aria-hidden="true">
		<span class="size-8 shrink-0 rounded-sm bg-surface-2"></span>
		<span class="flex flex-1 flex-col gap-1.5">
			<span class="h-2.5 w-35 rounded-2xs bg-surface-2"></span>
			<span class="h-2 w-full rounded-2xs bg-surface"></span>
		</span>
	</div>
	<span class="font-mono text-label text-text-3">{label}</span>
</div>
