<script lang="ts">
	import { ArrowRight, ArrowUp, ArrowUpRight, Mic, Plus, Trash2, Users } from '@lucide/svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import IconButton from '$lib/ds/actions/IconButton.svelte';
	import Link from '$lib/ds/actions/Link.svelte';
	import Kbd from '$lib/ds/actions/Kbd.svelte';
	import KeyHint from '$lib/ds/actions/KeyHint.svelte';
	import GallerySection from './GallerySection.svelte';
</script>

{#snippet specimen(caption: string, item: import('svelte').Snippet)}
	<div class="flex flex-col items-start gap-2">
		{@render item()}
		<span class="font-mono text-label text-text-3">{caption}</span>
	</div>
{/snippet}

<GallerySection
	title="Actions"
	description="One primary per view. Inverse for the main action inside panels, so mint stays rare. Height 30, radius-sm, 13px / 500."
>
	<div class="flex flex-col gap-3 rounded-md border border-border p-5">
		<p class="text-ui font-medium text-text">Button <span class="font-mono text-label text-text-3">variants</span></p>
		<div class="flex flex-wrap items-start gap-6">
			{#snippet primary()}<Button label="Start building" iconEnd={ArrowRight} />{/snippet}
			{#snippet inverse()}<Button label="Open workspace" variant="inverse" iconEnd={ArrowRight} />{/snippet}
			{#snippet secondary()}<Button label="Add credits on DeepSeek" variant="secondary" iconEnd={ArrowUpRight} />{/snippet}
			{#snippet ghost()}<Button label="Team and invites" variant="ghost" iconStart={Users} />{/snippet}
			{#snippet danger()}<Button label="Delete project" variant="danger" iconStart={Trash2} />{/snippet}
			{@render specimen('Primary', primary)}
			{@render specimen('Inverse', inverse)}
			{@render specimen('Secondary', secondary)}
			{@render specimen('Ghost', ghost)}
			{@render specimen('Danger', danger)}
		</div>
	</div>

	<div class="flex flex-col gap-3 rounded-md border border-border p-5">
		<p class="text-ui font-medium text-text">
			Button states <span class="font-mono text-label text-text-3">hover · focus-visible · active · disabled · loading</span>
		</p>
		<div class="flex flex-wrap items-start gap-6">
			{#snippet primaryHover()}<Button label="Start building" iconEnd={ArrowRight} data-hover />{/snippet}
			{#snippet primaryFocus()}<Button label="Start building" iconEnd={ArrowRight} data-focus />{/snippet}
			{#snippet secondaryHover()}<Button label="Add credits" variant="secondary" data-hover />{/snippet}
			{#snippet secondaryFocus()}<Button label="Add credits" variant="secondary" data-focus />{/snippet}
			{#snippet secondaryActive()}<Button label="Add credits" variant="secondary" data-active />{/snippet}
			{#snippet primaryDisabled()}<Button label="Start building" disabled />{/snippet}
			{#snippet inverseLoading()}<Button label="Deploying · 00:14" variant="inverse" loading />{/snippet}
			{@render specimen('Primary / Hover', primaryHover)}
			{@render specimen('Primary / Focus', primaryFocus)}
			{@render specimen('Secondary / Hover', secondaryHover)}
			{@render specimen('Secondary / Focus', secondaryFocus)}
			{@render specimen('Secondary / Active', secondaryActive)}
			{@render specimen('Primary / Disabled', primaryDisabled)}
			{@render specimen('Inverse / Loading', inverseLoading)}
		</div>
	</div>

	<div class="flex flex-col gap-3 rounded-md border border-border p-5">
		<p class="text-ui font-medium text-text">
			Icon button, link, kbd <span class="font-mono text-label text-text-3">30×30 hit area, 16px icon</span>
		</p>
		<div class="flex flex-wrap items-start gap-6">
			{#snippet outline()}<IconButton icon={Plus} label="Add" />{/snippet}
			{#snippet ghostIcon()}<IconButton icon={Mic} label="Record a voice note" variant="ghost" />{/snippet}
			{#snippet send()}<IconButton icon={ArrowUp} label="Send" variant="send" />{/snippet}
			{#snippet sendDisabled()}<IconButton icon={ArrowUp} label="Send" variant="send" disabled />{/snippet}
			{#snippet link()}<Link href="/settings/plan" label="Upgrade to Pro" />{/snippet}
			{#snippet inline()}<Link href="/settings/credits" label="switch to Nomi credits" variant="inline" />{/snippet}
			{#snippet kbd()}<Kbd keys="⌘K" />{/snippet}
			{#snippet hint()}<KeyHint
					hints={[
						{ keys: '↵', action: 'send' },
						{ keys: '⇧↵', action: 'new line' },
						{ keys: '@', action: 'address crew' },
					]}
				/>{/snippet}
			{@render specimen('Icon / Outline', outline)}
			{@render specimen('Icon / Ghost', ghostIcon)}
			{@render specimen('Icon / Send', send)}
			{@render specimen('Icon / Send / Disabled', sendDisabled)}
			{@render specimen('Link', link)}
			{@render specimen('Link / Inline', inline)}
			{@render specimen('Kbd', kbd)}
			{@render specimen('Key hint', hint)}
		</div>
	</div>
</GallerySection>
