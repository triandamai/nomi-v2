<script lang="ts">
	import { Check, Copy, ExternalLink, Globe, Share } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import Status from '../status/Status.svelte';
	import { createCopier } from '../actions/copier.svelte';
	import PhoneButton from './PhoneButton.svelte';

	// Mobile / Live Card: the build is live. The link to copy in one tap, Open and Share as big
	// buttons, and what the build took. A long link is clipped at the copy button, as drawn.
	type Props = { meta: string; url: string; summary: string; onshare: () => void };
	let { meta, url, summary, onshare }: Props = $props();

	const copier = createCopier();
</script>

<section class="flex w-full flex-col gap-3 rounded-3xl bg-surface p-4 outline -outline-offset-1 outline-border">
	<p class="flex items-center gap-2">
		<Mascot agent="koda" size={20} />
		<span class="text-phone-sm font-medium text-text">Koda</span>
		<span class="min-w-0 flex-1 font-mono text-label text-text-3">{meta}</span>
		<Status kind="live" label={m.build_live()} />
	</p>
	<div class="flex h-11 items-center gap-2 rounded-lg bg-surface-2 pr-1.5 pl-3">
		<span class="inline-flex shrink-0 text-text-2"><Icon icon={Globe} size={14} /></span>
		<span class="min-w-0 flex-1 overflow-hidden font-mono text-ui whitespace-nowrap text-text">{url.replace(/^https?:\/\//, '')}</span>
		<button
			type="button"
			aria-label={copier.copied ? m.copy_done() : m.build_copy_link()}
			class="inline-flex size-10 shrink-0 cursor-pointer items-center justify-center rounded-inner text-text outline-offset-1"
			onclick={() => copier.copy(url)}><Icon icon={copier.copied ? Check : Copy} /></button
		>
	</div>
	<div class="flex gap-2">
		<PhoneButton label={m.build_open()} icon={ExternalLink} href={url} external />
		<PhoneButton label={m.reply_share()} variant="secondary" icon={Share} onclick={onshare} />
	</div>
	<p class="text-caption text-text-3">{summary}</p>
</section>
