<script lang="ts">
	import { AtSign, ChevronRight, Copy, Check, Image, Mail, MessageCircle, Send, Users, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import IconButton from '../actions/IconButton.svelte';
	import Segmented from '../inputs/Segmented.svelte';
	import SectionLabel from '../status/SectionLabel.svelte';
	import { createCopier } from '../actions/copier.svelte';

	// Build → Share: the live link to copy, how it looks when shared (with a way to edit that), who
	// can open it, an invitation to build together, the apps to send it to, and more ways (QR,
	// embed). Reached from the live card; BuildShareDialog shows it as a modal.
	type Props = {
		titleId?: string;
		url: string;
		preview: { title: string; description?: string; image?: string };
		visibility?: 'public' | 'unlisted';
		onvisibility?: (visibility: 'public' | 'unlisted') => void;
		oneditpreview?: () => void;
		oninvite?: () => void;
		onmore?: () => void;
		onclose: () => void;
		ondone: () => void;
	};
	let { titleId, url, preview, visibility = 'public', onvisibility, oneditpreview, oninvite, onmore, onclose, ondone }: Props = $props();

	const copier = createCopier();
	const host = $derived(url.replace(/^https?:\/\//, ''));
	const link = $derived(encodeURIComponent(url));
	const apps = $derived([
		{ label: 'WhatsApp', icon: MessageCircle, href: `https://wa.me/?text=${link}` },
		{ label: 'Telegram', icon: Send, href: `https://t.me/share/url?url=${link}` },
		{ label: 'X', icon: AtSign, href: `https://x.com/intent/post?url=${link}` },
		{ label: m.bshare_email(), icon: Mail, href: `mailto:?body=${link}` },
	]);
</script>

<section class="flex w-120 max-w-full flex-col overflow-hidden rounded-md border border-border-strong bg-surface">
	<header class="flex items-center gap-2.5 border-b border-border px-5 py-4">
		<h2 id={titleId} class="min-w-0 flex-1 text-body font-medium text-text">{m.share_title()}</h2>
		<IconButton icon={X} label={m.action_close()} variant="ghost" onclick={onclose} />
	</header>
	<div class="flex flex-col gap-4 p-5">
		<div class="flex items-center gap-2">
			<span class="flex h-8.5 min-w-0 flex-1 items-center rounded-sm border border-border-strong bg-bg px-2.5">
				<span class="truncate font-mono text-label text-text">{host}</span>
			</span>
			<Button label={m.build_copy_link()} variant="secondary" iconEnd={copier.copied ? Check : Copy} onclick={() => copier.copy(url)} />
		</div>
		<div class="flex overflow-hidden rounded-sm border border-border">
			{#if preview.image}
				<img src={preview.image} alt="" class="h-24 w-30 shrink-0 object-cover" />
			{:else}
				<span class="inline-flex h-24 w-30 shrink-0 items-center justify-center bg-surface-2 text-text-3"><Icon icon={Image} size={20} /></span>
			{/if}
			<span class="flex min-w-0 flex-1 flex-col gap-1 px-3.5 py-3">
				<span class="truncate font-mono text-label text-text-3">{host}</span>
				<span class="text-ui font-medium text-text">{preview.title}</span>
				{#if preview.description}<span class="text-caption text-text-3">{preview.description}</span>{/if}
				{#if oneditpreview}
					<button type="button" class="cursor-pointer self-start rounded-xs text-caption text-text-2 outline-offset-1 hover:text-text" onclick={oneditpreview}>{m.bshare_edit_preview()}</button>
				{/if}
			</span>
		</div>
		<div class="flex items-center gap-3">
			<span class="flex min-w-0 flex-1 flex-col gap-0.5">
				<span class="text-ui font-medium text-text">{m.bshare_who()}</span>
				<span class="text-caption text-text-3">{m.bshare_who_help()}</span>
			</span>
			<Segmented
				surface="bg"
				label={m.bshare_who()}
				value={visibility}
				options={[
					{ value: 'public', label: m.bshare_public() },
					{ value: 'unlisted', label: m.bshare_unlisted() },
				]}
				onchange={(value) => onvisibility?.(value as 'public' | 'unlisted')}
			/>
		</div>
		{#if oninvite}
			<div class="flex items-center gap-3 rounded-sm border border-border px-3 py-2.5">
				<span class="inline-flex shrink-0 text-text"><Icon icon={Users} size={14} /></span>
				<span class="flex min-w-0 flex-1 flex-col gap-0.5">
					<span class="text-ui font-medium text-text">{m.bshare_together()}</span>
					<span class="text-caption text-text-3">{m.bshare_together_help({ name: 'Koda' })}</span>
				</span>
				<Button label={m.bshare_invite()} variant="secondary" onclick={oninvite} />
			</div>
		{/if}
		<div class="flex flex-col gap-2">
			<SectionLabel label={m.bshare_send_to()} />
			<div class="flex flex-wrap gap-2">
				{#each apps as app (app.label)}
					<a
						href={app.href}
						target={app.href.startsWith('mailto:') ? undefined : '_blank'}
						rel="noopener noreferrer"
						class="inline-flex h-7.5 items-center gap-1.5 rounded-sm border border-border-strong px-2.5 text-caption text-text outline-offset-1 hover:bg-surface-2 pointer-coarse:min-h-11"
					>
						<span class="inline-flex text-text-2"><Icon icon={app.icon} size={14} /></span>{app.label}
					</a>
				{/each}
			</div>
		</div>
		{#if onmore}
			<button type="button" class="flex cursor-pointer items-center gap-2 border-t border-border pt-2.5 text-left outline-offset-2" onclick={onmore}>
				<span class="inline-flex text-text-2"><Icon icon={ChevronRight} size={14} /></span>
				<span class="text-ui text-text-2">{m.bshare_more()}</span>
				<span class="flex-1 text-right font-mono text-label text-text-3">{m.bshare_more_meta()}</span>
			</button>
		{/if}
	</div>
	<footer class="flex justify-end border-t border-border px-5 py-3">
		<Button label={m.action_done()} variant="inverse" onclick={ondone} />
	</footer>
</section>
