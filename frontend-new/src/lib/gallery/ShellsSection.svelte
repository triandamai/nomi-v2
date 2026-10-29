<script lang="ts">
	import GallerySection from './GallerySection.svelte';
</script>

<!--
	As the pen draws the Shell Row: each diagram 220 tall in border-strong, 8px in; its regions square,
	surface-filled and edge to edge, each labelled top-left in text-2 on one line, cut off where the
	region ends, at the pen's sizes (sidebar 62, top and project bars 22, panel 90, agent chat 110,
	focus column 150).
-->
{#snippet region(label: string, classes: string)}
	<div class={['flex flex-col overflow-hidden border border-border bg-surface p-2 whitespace-nowrap', classes]}>
		<span class="font-mono text-label text-text-2">{label}</span>
	</div>
{/snippet}

<GallerySection
	title="Layout shells"
	description="All screens are 1512 × 862 at 1×. Columns are fixed; content is fill. Pick a shell before designing a screen."
>
	<div class="grid gap-6 md:grid-cols-3">
		<div class="flex min-w-0 flex-col gap-2.5">
			<div data-diagram class="flex h-55 rounded-md border border-border-strong p-2">
				{@render region('Sidebar 248', 'w-15.5 shrink-0')}
				<div class="flex min-w-0 flex-1 flex-col">
					{@render region('Top bar 52', 'h-5.5 shrink-0')}
					{@render region('Content · pad 32–56', 'min-h-0 flex-1')}
				</div>
				{@render region('Panel 300–380', 'w-22.5 shrink-0')}
			</div>
			<p class="text-ui font-medium text-text">App shell</p>
			<p class="text-caption text-text-3">Home, Chats, Projects, Money…</p>
		</div>
		<div class="flex min-w-0 flex-col gap-2.5">
			<div data-diagram class="flex h-55 flex-col gap-1.5 rounded-md border border-border-strong p-2">
				{@render region('Project bar 52', 'h-5.5 shrink-0')}
				<div class="flex min-h-0 flex-1">
					{@render region('Agent chat 440', 'w-27.5 shrink-0')}
					{@render region('Preview / Code / Terminal', 'min-w-0 flex-1')}
				</div>
			</div>
			<p class="text-ui font-medium text-text">Workspace shell</p>
			<p class="text-caption text-text-3">Inside one project</p>
		</div>
		<div class="flex min-w-0 flex-col gap-2.5">
			<div data-diagram class="flex h-55 justify-center rounded-md border border-border-strong p-2">
				{@render region('Column 480–720', 'w-37.5 shrink-0')}
			</div>
			<p class="text-ui font-medium text-text">Focus shell</p>
			<p class="text-caption text-text-3">Onboarding, dialogs, checkout</p>
		</div>
	</div>
</GallerySection>
