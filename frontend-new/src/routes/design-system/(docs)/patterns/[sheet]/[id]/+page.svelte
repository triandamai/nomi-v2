<script lang="ts">
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import SheetEmbed from '$lib/docs/ui/SheetEmbed.svelte';
	import Guidance from '$lib/docs/ui/Guidance.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import LinkCard from '$lib/docs/ui/LinkCard.svelte';
	import { componentDocs } from '$lib/docs/entries';
	import { SHEETS } from '$lib/docs/patterns';
	import { docHref } from '$lib/docs/href';

	let { data } = $props();

	// A pattern: how components combine for one job, shown as its sheet draws it, with the rules.
</script>

{#key `${data.pattern.sheet}/${data.pattern.id}`}
	<DocPage eyebrow="Pattern · {SHEETS[data.pattern.sheet]}" title={data.pattern.title} lede={data.doc?.summary} wide>
		{#if data.doc?.use?.length}
			<DocSection title="When it comes up"><Guidance use={data.doc.use} /></DocSection>
		{/if}
		<DocSection title="On the sheet" lede="The real components, laid out exactly as the pen's section. Scroll sideways on smaller screens.">
			<SheetEmbed sheet={data.pattern.sheet} id={data.pattern.id} label={data.pattern.title}>
				<data.Section />
			</SheetEmbed>
		</DocSection>
		{#if data.doc?.dos?.length || data.doc?.donts?.length}
			<DocSection title="Best practices"><DoDont dos={data.doc.dos} donts={data.doc.donts} /></DocSection>
		{/if}
		{#if data.doc?.content?.length}
			<DocSection title="Writing for it">
				<ul>
					{#each data.doc.content as item (item)}<li>{item}</li>{/each}
				</ul>
			</DocSection>
		{/if}
		{#if data.used.length}
			<DocSection title="Components in it" lede="Each has its own page with props and usage.">
				<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
					{#each data.used as item (item.file)}
						<LinkCard href={docHref(`/design-system/components/${item.slug}`)} title={item.name} blurb={componentDocs[item.file]?.summary} />
					{/each}
				</div>
			</DocSection>
		{/if}
	</DocPage>
{/key}
