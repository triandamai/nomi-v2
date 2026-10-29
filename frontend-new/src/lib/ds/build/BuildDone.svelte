<script lang="ts">
	import { ArrowUpRight, Check, ChevronDown, Copy, Globe, Share2 } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import Status from '../status/Status.svelte';
	import { createCopier } from '../actions/copier.svelte';

	// Build → Done: the live link to copy or open, what the build actually took (time, allowance),
	// the details on request, and what's next: share it; later changes need Publish.
	type Props = { meta: string; url: string; summary: string; ondetails?: () => void; onshare?: () => void };
	let { meta, url, summary, ondetails, onshare }: Props = $props();

	const copier = createCopier();
</script>

<section class="flex w-full flex-col overflow-hidden rounded-md border border-border bg-surface">
	<header class="flex items-center gap-2.5 border-b border-border px-4 py-3.5">
		<Mascot agent="koda" size={20} />
		<span class="text-ui font-medium text-text">Koda</span>
		<span class="min-w-0 flex-1 font-mono text-label text-text-3">{meta}</span>
		<Status kind="live" label={m.build_live()} small />
	</header>
	<div class="flex flex-wrap items-center gap-2.5 px-4 py-3.5">
		<span class="inline-flex shrink-0 text-text-2"><Icon icon={Globe} size={14} /></span>
		<span class="min-w-0 flex-1 truncate font-mono text-ui font-medium text-text">{url.replace(/^https?:\/\//, '')}</span>
		<span class="flex items-center gap-1">
			<button
				type="button"
				aria-label={copier.copied ? m.copy_done() : m.build_copy_link()}
				class="inline-flex h-7.5 cursor-pointer items-center rounded-sm px-2 text-text-3 outline-offset-1 hover:bg-surface-2 hover:text-text pointer-coarse:min-h-11"
				onclick={() => copier.copy(url)}><Icon icon={copier.copied ? Check : Copy} size={14} /></button
			>
			<Button label={m.build_open()} variant="ghost" iconStart={ArrowUpRight} href={url} external />
		</span>
	</div>
	<div class="flex items-center gap-2 px-4 pb-3">
		<span class="min-w-0 flex-1 text-caption text-text-2">{summary}</span>
		{#if ondetails}
			<button type="button" class="inline-flex cursor-pointer items-center gap-1 rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text pointer-coarse:min-h-11" onclick={ondetails}>
				{m.build_details()}<span class="inline-flex text-text-3"><Icon icon={ChevronDown} size={12} /></span>
			</button>
		{/if}
	</div>
	<div class="flex flex-wrap items-center gap-2 px-4 py-3">
		{#if onshare}<Button label={m.reply_share()} variant="secondary" iconEnd={Share2} onclick={onshare} />{/if}
		<span class="min-w-0 flex-1 text-right text-caption text-text-3">{m.build_after_publish()}</span>
	</div>
</section>
