<script lang="ts">
	import { FileText, Globe } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import SectionLabel from '../status/SectionLabel.svelte';
	import Citation from './Citation.svelte';

	// Chat responses → Sources: what an answer leaned on, numbered like its citations, each with
	// where it came from (an attachment and who read it, or a site's domain). Each item's id is
	// `<id>-<n>`, so a Citation can link to it.
	type Source = { title: string; detail?: string; kind: 'file' | 'web'; href?: string };
	let { id, sources }: { id: string; sources: Source[] } = $props();
</script>

<section class="flex w-full flex-col gap-2">
	<SectionLabel label={m.sources_title()} />
	<ol class="flex flex-col gap-2">
		{#each sources as source, index (index)}
			<li id="{id}-{index + 1}" class="flex min-w-0 items-start gap-2">
				<Citation n={index + 1} title={source.title} />
				<span class="inline-flex h-4 shrink-0 items-center text-text-3"><Icon icon={source.kind === 'file' ? FileText : Globe} size={12} /></span>
				<span class="flex min-w-0 flex-1 flex-wrap items-baseline gap-x-2">
					{#if source.href}
						<a
							href={source.href}
							target="_blank"
							rel="noopener noreferrer"
							class="min-w-0 truncate rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text hover:underline hover:decoration-border-strong hover:underline-offset-2">{source.title}</a
						>
					{:else}
						<span class="min-w-0 truncate text-caption text-text-2">{source.title}</span>
					{/if}
					{#if source.detail}<span class="min-w-0 truncate font-mono text-label text-text-3">{source.detail}</span>{/if}
				</span>
			</li>
		{/each}
	</ol>
</section>
