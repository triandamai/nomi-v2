<script lang="ts">
	import GallerySection from './GallerySection.svelte';
</script>

{#snippet column(label: string, classes: string)}
	<div class={['flex items-center justify-center rounded-xs border border-border-strong bg-surface p-2 text-center font-mono text-label text-text-3', classes]}>{label}</div>
{/snippet}

<GallerySection
	title="Layout shells"
	description="All screens are 1512 × 862 at 1×. Columns are fixed; content is fill. Pick a shell before designing a screen."
>
	<div class="grid grid-cols-3 gap-6">
		<div class="flex flex-col gap-2">
			<div class="flex h-60 gap-1 rounded-md border border-border p-1">
				{@render column('Sidebar 248', 'w-16')}
				<div class="flex flex-1 flex-col gap-1">
					{@render column('Top bar 52', 'h-8')}
					{@render column('Content · pad 32–56', 'flex-1')}
				</div>
				{@render column('Panel 300–380', 'w-20')}
			</div>
			<p class="text-ui font-medium text-text">App shell</p>
			<p class="text-caption text-text-3">Home, Chats, Projects, Money…</p>
		</div>
		<div class="flex flex-col gap-2">
			<div class="flex h-60 flex-col gap-1 rounded-md border border-border p-1">
				{@render column('Project bar 52', 'h-8')}
				<div class="flex flex-1 gap-1">
					{@render column('Agent chat 440', 'w-28')}
					{@render column('Preview / Code / Terminal', 'flex-1')}
				</div>
			</div>
			<p class="text-ui font-medium text-text">Workspace shell</p>
			<p class="text-caption text-text-3">Inside one project</p>
		</div>
		<div class="flex flex-col gap-2">
			<div class="flex h-60 justify-center rounded-md border border-border p-1">
				{@render column('Column 480–720', 'w-36')}
			</div>
			<p class="text-ui font-medium text-text">Focus shell</p>
			<p class="text-caption text-text-3">Onboarding, dialogs, checkout</p>
		</div>
	</div>
</GallerySection>
