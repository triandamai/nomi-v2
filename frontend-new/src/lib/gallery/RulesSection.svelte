<script lang="ts">
	import { Check, X } from '@lucide/svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import GallerySection from './GallerySection.svelte';

	const rules = [
		{
			title: 'Do',
			icon: Check,
			items: [
				'Use nm-* tokens for every colour, size, gap and radius',
				'Mono for paths, IDs, URLs, tokens, money, timestamps',
				'One accent action per view; Inverse for panel actions',
				'Show elapsed time + current step when duration is unknown',
				'Put Export next to anything that looks like lock-in',
				"Use the agent's mascot when that agent speaks or works",
				'Ship empty, loading, error and populated states together',
			],
		},
		{
			title: "Don't",
			icon: X,
			items: [
				'Gradients, glass, glows, or a second accent hue',
				'Emoji or illustrations in UI chrome',
				'Shadows on flat cards or rows',
				'Progress bars that creep toward 90%',
				'"Magic", "effortless", "AI-powered" badges',
				'Title Case or ALL CAPS outside the mono label',
				'Pure #000 or #FFF surfaces',
			],
		},
	];
</script>

<GallerySection title="Rules" description="Checkable. If a frame breaks one, fix it before showing it.">
	<div class="grid items-start gap-6 md:grid-cols-2">
		{#each rules as list (list.title)}
			<section aria-label={list.title} class="flex flex-col overflow-hidden rounded-md outline -outline-offset-1 outline-border">
				<p class="flex items-center gap-2 border-b border-border bg-surface px-4 pt-2.5 pb-2.25 text-ui font-medium text-text">
					<span class={['inline-flex', list.icon === Check ? 'text-accent' : 'text-danger']}><Icon icon={list.icon} size={14} /></span>{list.title}
				</p>
				<ul>
					{#each list.items as rule (rule)}<li class="border-b border-border px-4 pt-2.25 pb-2 text-ui text-text-2">{rule}</li>{/each}
				</ul>
			</section>
		{/each}
	</div>
</GallerySection>
