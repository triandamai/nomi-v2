<script lang="ts">
	import GallerySection from './GallerySection.svelte';

	const crew = ['Nomi', 'Koda', 'Dana', 'Kala', 'Rena', 'Tara', 'Maya', 'Raga', 'Personality', 'Supervisor'];
	const useFor = [
		'Cards and tiles that are a crew member (crew header, selected member). Suggestions and template cards tint only on hover',
		'Who an item is from (inbox source tile, today panels, suggestions)',
		'Per-member data (usage bars, breakdowns)',
		'App identity tiles on projects',
	];
	const neverFor = [
		'Buttons, links, focus or live state — that’s nm-accent',
		'Navigation, settings rows, forms, tables',
		'Errors and warnings — keep danger / warning',
		'Large page backgrounds inside the app',
	];
</script>

<GallerySection
	title="Crew colour"
	description="Each crew member owns one colour. Use it only on surfaces that belong to that member. App chrome stays neutral; nm-accent stays the only action and live colour."
>
	<div class="grid grid-cols-5 gap-3">
		{#each crew as name (name)}
			{@const key = name.toLowerCase()}
			<div class="flex flex-col gap-2 rounded-md border border-border bg-surface p-3">
				<p class="text-ui font-medium" style="color: var(--crew-{key}-ink)">{name}</p>
				{#each [`crew-${key}`, `crew-${key}-bg`, `crew-${key}-ink`] as token (token)}
					<div class="flex items-center gap-2">
						<span data-token={token} class="size-4 shrink-0 rounded-sm border border-border" style="background: var(--{token})"></span>
						<span class="font-mono text-label text-text-2">{token}</span>
					</div>
				{/each}
			</div>
		{/each}
	</div>
	<div class="grid grid-cols-2 gap-6">
		{#each [{ title: 'Use it for', items: useFor }, { title: 'Never for', items: neverFor }] as list (list.title)}
			<div class="flex flex-col gap-2">
				<p class="font-mono text-label tracking-label text-text-3 uppercase">{list.title}</p>
				<ul class="flex flex-col gap-1 text-caption text-text-2">
					{#each list.items as item (item)}<li>· {item}</li>{/each}
				</ul>
			</div>
		{/each}
	</div>
</GallerySection>
