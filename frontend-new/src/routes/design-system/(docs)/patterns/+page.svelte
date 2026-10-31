<script lang="ts">
	import DocPage from '$lib/docs/ui/DocPage.svelte';
	import DocSection from '$lib/docs/ui/DocSection.svelte';
	import LinkCard from '$lib/docs/ui/LinkCard.svelte';
	import { PATTERNS, SHEETS } from '$lib/docs/patterns';
	import { patternDocs } from '$lib/docs/pattern-docs';
	import { docHref } from '$lib/docs/href';

	const sheets = ['chat', 'build'] as const;
	const ledes = {
		chat: 'What a reply can hold, and the controls around a conversation: thinking, rich content, approvals, feedback, errors, long threads, the crew, and chat on a phone.',
		build: 'From describing an app to a live URL: the estimate, the build, sharing, the workspace and its tools, building together, and building from a phone.',
	};
</script>

<DocPage
	eyebrow="Patterns"
	title="Patterns"
	lede="How components combine for one job. Each pattern is a section of the Chat responses or Build sheet, shown with the real components exactly as the pen draws it, with the rules that go with it."
	wide
>
	{#each sheets as sheet (sheet)}
		<DocSection title={SHEETS[sheet]} id={sheet} lede={ledes[sheet]}>
			<div class="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
				{#each PATTERNS.filter((pattern) => pattern.sheet === sheet) as pattern (pattern.id)}
					<LinkCard href={docHref(`/design-system/patterns/${sheet}/${pattern.id}`)} title={pattern.title} blurb={patternDocs[`${sheet}/${pattern.id}`]?.summary} />
				{/each}
			</div>
		</DocSection>
	{/each}
</DocPage>
