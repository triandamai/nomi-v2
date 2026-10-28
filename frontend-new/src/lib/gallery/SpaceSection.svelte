<script lang="ts">
	import { House } from '@lucide/svelte';
	import Icon from '$lib/ds/foundations/Icon.svelte';
	import GallerySection from './GallerySection.svelte';

	const iconSizes = [16, 14, 12] as const;
	const spacing: [step: string, use: string][] = [
		['2xs', '2px · nav item stack'],
		['xs', '4px · label to value'],
		['sm', '6px · icon to label'],
		['md', '8px · chip groups'],
		['lg', '12px · row content'],
		['xl', '16px · panel padding'],
		['2xl', '20px · panel side pad'],
		['3xl', '24px · section gap'],
		['4xl', '32px · page padding'],
		['5xl', '48px · page header gap'],
	];
</script>

<GallerySection title="Spacing" description="4px base with 2 / 6 for dense UI. Gaps and padding only from this scale.">
	<div class="flex flex-col gap-2">
		{#each spacing as [step, use] (step)}
			<div class="flex items-center gap-4">
				<span class="w-24 font-mono text-label text-text">space-{step}</span>
				<span class="h-3 rounded-sm bg-accent" style="width: var(--nm-space-{step})"></span>
				<span class="font-mono text-label text-text-3">{use}</span>
			</div>
		{/each}
	</div>
</GallerySection>

<GallerySection title="Radius & depth" description="Two radii. Borders define edges; one shadow level, only for floating layers.">
	<div class="flex gap-6">
		<div class="flex flex-col gap-2">
			<div class="size-20 rounded-sm border border-border-strong bg-surface"></div>
			<span class="font-mono text-label text-text">radius-sm · 6px</span>
			<span class="text-caption text-text-3">buttons, chips, rows, inputs</span>
		</div>
		<div class="flex flex-col gap-2">
			<div class="size-20 rounded-md border border-border-strong bg-surface"></div>
			<span class="font-mono text-label text-text">radius-md · 10px</span>
			<span class="text-caption text-text-3">panels, composer, callouts</span>
		</div>
		<div class="flex flex-col gap-2">
			<div class="size-20 rounded-md border border-border bg-surface"></div>
			<span class="font-mono text-label text-text-3">Flat + border</span>
		</div>
		<div class="flex flex-col gap-2">
			<div class="size-20 rounded-md border border-border-strong bg-surface shadow-float"></div>
			<span class="font-mono text-label text-text-3">Floating</span>
		</div>
	</div>
</GallerySection>

<GallerySection title="Iconography" description="Lucide only, 1.5 stroke. 16px in nav and buttons, 14px inline with text, 12px in chips.">
	<div class="flex items-end gap-6 text-text-2">
		{#each iconSizes as size (size)}
			<div class="flex flex-col items-center gap-2">
				<Icon icon={House} {size} />
				<span class="font-mono text-label text-text-3">{size}px</span>
			</div>
		{/each}
	</div>
</GallerySection>
