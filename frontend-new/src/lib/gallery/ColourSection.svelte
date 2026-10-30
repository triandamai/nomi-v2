<script lang="ts">
	import GallerySection from './GallerySection.svelte';

	// The design sheet's colour groups, in its order and with its words. The support colours the
	// other sheets use (heat, code, diff, keyboard, peers) aren't drawn here, as in the pen.
	const groups: { title: string; tokens: [name: string, use: string][] }[] = [
		{
			title: 'Surfaces',
			tokens: [
				['nm-bg', 'Page, sidebars'],
				['nm-surface', 'Cards, composer, light main'],
				['nm-surface-2', 'Active row, chips, bubbles'],
			],
		},
		{
			title: 'Lines',
			tokens: [
				['nm-border', 'Dividers, quiet outlines'],
				['nm-border-strong', 'Inputs, buttons, emphasis'],
			],
		},
		{
			title: 'Text',
			tokens: [
				['nm-text', 'Primary text, inverse buttons'],
				['nm-text-2', 'Secondary, body in threads'],
				['nm-text-3', 'Meta, placeholders, mono labels'],
			],
		},
		{
			title: 'Accent',
			tokens: [
				['nm-accent', 'Send, live dot, logo'],
				['nm-accent-ink', 'Content on accent'],
				['nm-accent-bg', 'Accent tint for states'],
			],
		},
		{
			title: 'Semantic',
			tokens: [
				['nm-danger', 'Errors, no-go'],
				['nm-danger-bg', 'Error header tint'],
				['nm-danger-border', 'Error tag outline'],
				['nm-warning', 'Needs attention'],
				['nm-warning-bg', 'Warning tint'],
			],
		},
		{
			title: 'Utility',
			tokens: [
				['nm-focus', 'Focus ring'],
				['nm-overlay', 'Dialog scrim'],
				['nm-shadow', 'Floating shadow'],
			],
		},
	];
</script>

<GallerySection
	title="Colour"
	description="13 base tokens + 6 support tokens. Each resolves per theme; never hardcode a hex in product UI."
>
	<div class="grid gap-6 md:grid-cols-2">
		{#each ['dark', 'light'] as theme (theme)}
			<div data-theme={theme} class="flex min-w-0 flex-col gap-5 rounded-md border border-border bg-bg p-6">
				<p class="font-mono text-label tracking-label text-text uppercase">{theme}</p>
				{#each groups as group (group.title)}
					<div class="flex flex-col gap-1.5">
						<p class="font-mono text-label tracking-label text-text-3 uppercase">{group.title}</p>
						{#each group.tokens as [name, use] (name)}
							<div class="flex items-center gap-3">
								<span data-token={name} class="h-6 w-10 shrink-0 rounded-sm border border-border-strong" style="background: var(--{name})"></span>
								<span class="w-35 shrink-0 font-mono text-label text-text">{name}</span>
								<span class="min-w-0 text-caption text-text-3">{use}</span>
							</div>
						{/each}
					</div>
				{/each}
			</div>
		{/each}
	</div>
</GallerySection>
