<script lang="ts">
	import Search from '$lib/ds/inputs/Search.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import LinkCard from '$lib/docs/ui/LinkCard.svelte';
	import { CATEGORIES } from '$lib/docs/catalog';
	import { api, components } from '$lib/docs/components';
	import { componentDocs } from '$lib/docs/entries';
	import { docHref } from '$lib/docs/href';

	// Every component, by category, each with the line that says what it's for.
	let filter = $state('');
	const summary = (file: string) => componentDocs[file]?.summary ?? api[file].notes.split(/(?<=\.)\s/)[0];
	const shown = $derived(
		components.filter((item) => {
			const words = filter.toLowerCase().split(/\s+/).filter(Boolean);
			const text = `${item.name} ${summary(item.file)}`.toLowerCase();
			return words.every((word) => text.includes(word));
		}),
	);
</script>

<DocPage
	eyebrow="Components"
	title="Components"
	lede="Every component of the new frontend, from a button to a build plan. Each page shows it live, lists its props as the code declares them, and says when to use it."
	wide
>
	{#snippet meta()}
		<span class="font-mono text-label text-text-3">{components.length} components · {CATEGORIES.length} categories</span>
	{/snippet}
	<div class="max-w-90"><Search placeholder="Filter components" label="Filter components" bind:value={filter} /></div>
	{#each CATEGORIES as category (category.key)}
		{@const items = shown.filter((item) => item.category === category.key)}
		{#if items.length}
			<DocSection title={category.title} id={category.key} lede={category.blurb}>
				<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
					{#each items as item (item.file)}
						<LinkCard href={docHref(`/design-system/components/${item.slug}`)} title={item.name} blurb={summary(item.file)} />
					{/each}
				</div>
			</DocSection>
		{/if}
	{/each}
	{#if !shown.length}<p>Nothing matches “{filter}”. Try part of a name, like “menu”.</p>{/if}
</DocPage>
