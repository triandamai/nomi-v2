<script lang="ts">
	// When to reach for something, and when to reach for something else.
	let { use = [], avoid = [] }: { use?: string[]; avoid?: string[] } = $props();
	const columns = $derived([
		{ title: 'Use it', items: use },
		{ title: 'Use something else', items: avoid },
	].filter((column) => column.items.length));
</script>

<div class="grid gap-x-8 gap-y-6 md:grid-cols-2">
	{#each columns as column (column.title)}
		<div class="flex flex-col gap-2.5">
			<h3 class="text-ui font-medium text-text">{column.title}</h3>
			<ul class="flex flex-col gap-2 border-l border-border pl-4">
				{#each column.items as item (item)}
					<li class="text-ui leading-normal text-text-2">{item}</li>
				{/each}
			</ul>
		</div>
	{/each}
</div>
