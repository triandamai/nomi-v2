<script lang="ts">
	import Tabs from '$lib/ds/inputs/Tabs.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import CodeBlock from '$lib/ds/chat/CodeBlock.svelte';
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import Example from '$lib/docs/ui/Example.svelte';
	import PropsTable from '$lib/docs/ui/PropsTable.svelte';
	import Guidance from '$lib/docs/ui/Guidance.svelte';
	import DoDont from '$lib/docs/ui/DoDont.svelte';
	import LinkCard from '$lib/docs/ui/LinkCard.svelte';
	import Note from '$lib/docs/ui/Note.svelte';
	import { CATEGORIES } from '$lib/docs/catalog';
	import { components } from '$lib/docs/components';
	import { componentDocs } from '$lib/docs/entries';
	import { highlight } from '$lib/docs/highlight';
	import { docHref } from '$lib/docs/href';
	import type { SectionRef } from '$lib/docs/api/relations';

	let { data } = $props();

	// One component, as Atlassian's docs lay it out: Examples (live), Code (import, props from the
	// source, notes), Usage (when, rules, writing, accessibility). Every page has all three.
	const category = $derived(CATEGORIES.find((candidate) => candidate.key === data.item.category)!);
	const importLine = $derived(`import ${data.item.name} from '$lib/ds/${data.item.file}';`);
	const notes = $derived(data.api.notes.split('\n\n').filter(Boolean));
	const lede = $derived(data.doc?.summary ?? notes[0] ?? '');
	const sheetNames = { nomi: 'Nomi', chat: 'Chat responses', build: 'Build' };
	const sectionHref = (section: SectionRef) =>
		section.sheet === 'nomi' ? docHref('/design-system/nomi', section.id) : docHref(`/design-system/patterns/${section.sheet}/${section.id}`);
	const linked = (files: string[]) => files.map((file) => components.find((item) => item.file === file)).filter((item) => !!item);
	let tab = $state('examples');
</script>

{#snippet componentCards(files: string[])}
	<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
		{#each linked(files) as item (item.file)}
			<LinkCard href={docHref(`/design-system/components/${item.slug}`)} title={item.name} blurb={componentDocs[item.file]?.summary} />
		{/each}
	</div>
{/snippet}

{#snippet list(items: string[])}
	<ul>
		{#each items as item (item)}<li>{item}</li>{/each}
	</ul>
{/snippet}

{#key data.item.slug}
	<DocPage eyebrow={category.title} title={data.item.name} {lede} wide>
		{#snippet meta()}
			<Tag kind={data.api.sections.length ? 'go' : 'neutral'} label={data.api.sections.length ? 'On the pen' : 'Building block'} />
			<code class="font-mono text-label text-text-3">src/lib/ds/{data.item.file}</code>
		{/snippet}

		<div>
			<Tabs
				label="{data.item.name} documentation"
				bind:value={tab}
				tabs={[
					{ value: 'examples', label: 'Examples' },
					{ value: 'code', label: 'Code' },
					{ value: 'usage', label: 'Usage' },
				]}
			>
				{#snippet panel(value)}
					<div class="flex flex-col gap-12 pt-8">
						{#if value === 'examples'}
							{#each data.examples as example (example.title)}
								<Example title={example.title} description={example.description} code={example.code}>
									<example.component />
								</Example>
							{:else}
								<Note>
									<p>
										No standalone example yet.
										{#if data.api.sections.length}See it working in the sheets below, exactly as the pen draws it.{:else}It is a part of other components: see them under Code.{/if}
									</p>
								</Note>
							{/each}
							{#if data.api.sections.length}
								<DocSection title="In the pen" lede="The sheet sections that show it, drawn 1:1 with Nomi.pen.">
									<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
										{#each data.api.sections as section (`${section.sheet}/${section.id}`)}
											<LinkCard href={sectionHref(section)} eyebrow="{sheetNames[section.sheet]} sheet" title={section.title} />
										{/each}
									</div>
								</DocSection>
							{/if}
						{:else if value === 'code'}
							<DocSection title="Import">
								<CodeBlock code={importLine} language="ts" tokens={highlight(importLine, 'ts')} />
							</DocSection>
							<DocSection title="Props" lede="Read from the component's source when the docs are built, so they are always the code's.">
								<PropsTable api={data.api} />
							</DocSection>
							{#if notes.length}
								<DocSection title="Notes from the source" lede="What the code says about itself, including how it meets the pen.">
									{#each notes as note (note)}<p>{note}</p>{/each}
								</DocSection>
							{/if}
							{#if data.api.uses.length}
								<DocSection title="Built from">{@render componentCards(data.api.uses)}</DocSection>
							{/if}
							{#if data.api.usedBy.length}
								<DocSection title="Used by">{@render componentCards(data.api.usedBy)}</DocSection>
							{/if}
						{:else if data.doc}
							{#if data.doc.use?.length || data.doc.avoid?.length}
								<DocSection title="When to use it"><Guidance use={data.doc.use} avoid={data.doc.avoid} /></DocSection>
							{/if}
							{#if data.doc.dos?.length || data.doc.donts?.length}
								<DocSection title="Best practices"><DoDont dos={data.doc.dos} donts={data.doc.donts} /></DocSection>
							{/if}
							{#if data.doc.content?.length}
								<DocSection title="Writing for it">{@render list(data.doc.content)}</DocSection>
							{/if}
							{#if data.doc.accessibility?.length}
								<DocSection title="Accessibility">{@render list(data.doc.accessibility)}</DocSection>
							{/if}
							{#if data.doc.related?.length}
								<DocSection title="Related">{@render componentCards(data.doc.related)}</DocSection>
							{/if}
						{:else}
							<Note><p>Usage guidance for {data.item.name} isn't written yet. Until it is, follow the sheet it appears in and the notes under Code.</p></Note>
						{/if}
					</div>
				{/snippet}
			</Tabs>
		</div>
	</DocPage>
{/key}
