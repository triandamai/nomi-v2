<script lang="ts">
	import GallerySection from './GallerySection.svelte';

	const rules = [
		{
			title: 'Do',
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
	<div class="grid grid-cols-2 gap-6">
		{#each rules as list (list.title)}
			<div class="flex flex-col gap-2 rounded-md border border-border p-5">
				<p class="font-mono text-label tracking-label text-text-3 uppercase">{list.title}</p>
				<ul class="flex flex-col gap-1.5 text-ui text-text-2">
					{#each list.items as rule (rule)}<li>{rule}</li>{/each}
				</ul>
			</div>
		{/each}
	</div>
</GallerySection>
