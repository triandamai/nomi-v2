<script lang="ts">
	import { Check, ChevronRight, Copy, Ellipsis, Globe, Mail, MessageCircle, Send, Users } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import BottomSheet from '../overlays/BottomSheet.svelte';
	import { createCopier } from '../actions/copier.svelte';

	// Flow / Mobile Build → Share (CFzPR; a site's eMzUB): a live app's link from a phone. The address
	// with Copy, then (for a site) its card and whether it's on show, Build together (Invite people to
	// prompt Koda with you) where the person may invite, and where to send it as 52 round tiles:
	// WhatsApp, Telegram, Mail, and More (the phone's own share, only where the browser has one). No
	// QR and no embed: nothing behind them.
	type Props = {
		open?: boolean;
		project: string;
		url: string;
		/** A site leads with its card: the project's name and description. */
		card?: { title: string; description?: string };
		/** Public (on show in the community) or unlisted; changeable where `onvisibility` is given. */
		visibility?: 'public' | 'unlisted';
		onvisibility?: (next: 'public' | 'unlisted') => void;
		oninvite?: () => void;
		onclose?: () => void;
	};
	let { open = $bindable(false), project, url, card, visibility, onvisibility, oninvite, onclose }: Props = $props();

	const copier = createCopier();
	const text = $derived(encodeURIComponent(url));
	const shown = $derived(url.replace(/^https?:\/\//, '').replace(/\/$/, ''));
	const canShare = $derived(typeof navigator !== 'undefined' && 'share' in navigator);
</script>

<BottomSheet bind:open title={m.pb_share_title({ name: project })} subtitle={m.pb_share_subtitle()} {onclose}>
	<div class="flex flex-col gap-3 px-4 pb-2">
		{#if card}
			<div class="flex flex-col gap-1 rounded-xl p-3.5 outline -outline-offset-1 outline-border">
				<span class="text-body font-medium text-text">{card.title}</span>
				{#if card.description}<span class="text-ui text-text-2">{card.description}</span>{/if}
				<span class="font-mono text-label text-text-3">{shown}</span>
			</div>
		{/if}
		<div class="flex h-12 items-center gap-2 rounded-lg bg-surface-2 pr-1.5 pl-3.5">
			<span class="min-w-0 flex-1 truncate font-mono text-ui text-text">{shown}</span>
			<button type="button" aria-label={copier.copied ? m.copy_done() : m.dns_copy({ value: shown })} class="inline-flex h-9 shrink-0 cursor-pointer items-center gap-1.5 rounded-inner border border-border-strong px-2.75 text-ui text-text outline-offset-1 pointer-coarse:min-h-11" onclick={() => copier.copy(url)}>
				<Icon icon={copier.copied ? Check : Copy} size={14} />{copier.copied ? m.copy_done() : m.pb_copy()}
			</button>
		</div>
		{#if visibility}
			<div role="radiogroup" aria-label={m.pb_visibility()} class="flex rounded-lg bg-surface-2 p-1">
				{#each [{ value: 'public', label: m.pb_public() }, { value: 'unlisted', label: m.bshare_unlisted() }] as const as option (option.value)}
					<button
						type="button"
						role="radio"
						aria-checked={visibility === option.value}
						disabled={!onvisibility}
						class={['h-9 flex-1 cursor-pointer rounded-inner text-phone-sm outline-offset-1 disabled:cursor-default pointer-coarse:min-h-11', visibility === option.value ? 'bg-surface font-medium text-text' : 'text-text-2']}
						onclick={() => onvisibility?.(option.value)}>{option.label}</button
					>
				{/each}
			</div>
		{/if}
		{#if oninvite}
			<button type="button" class="flex min-h-15 w-full cursor-pointer items-center gap-3 rounded-xl px-3.5 py-2 text-left outline -outline-offset-1 outline-border" onclick={oninvite}>
				<span class="inline-flex shrink-0 text-text-2"><Icon icon={Users} size={20} /></span>
				<span class="flex min-w-0 flex-1 flex-col gap-0.5">
					<span class="text-phone text-text">{m.bshare_together()}</span>
					<span class="text-ui text-text-3">{m.bshare_together_help({ name: 'Koda' })}</span>
				</span>
				<span class="inline-flex text-text-3"><Icon icon={ChevronRight} /></span>
			</button>
		{/if}
		<div class="flex justify-between gap-2 pt-1">
			{#snippet target(label: string, icon: typeof Globe)}
				<span class="inline-flex size-13 items-center justify-center rounded-2xl bg-surface-2 text-text"><Icon {icon} size={20} /></span>
				<span class="text-caption text-text-2">{label}</span>
			{/snippet}
			<a href="https://wa.me/?text={text}" target="_blank" rel="noopener noreferrer" class="flex w-15 flex-col items-center gap-1.5 outline-offset-2">{@render target('WhatsApp', MessageCircle)}</a>
			<a href="https://t.me/share/url?url={text}" target="_blank" rel="noopener noreferrer" class="flex w-15 flex-col items-center gap-1.5 outline-offset-2">{@render target('Telegram', Send)}</a>
			<a href="mailto:?body={text}" class="flex w-15 flex-col items-center gap-1.5 outline-offset-2">{@render target(m.share_mail(), Mail)}</a>
			{#if canShare}
				<button type="button" class="flex w-15 cursor-pointer flex-col items-center gap-1.5 outline-offset-2" onclick={() => navigator.share({ url }).catch(() => undefined)}>{@render target(m.mobile_more(), Ellipsis)}</button>
			{/if}
		</div>
	</div>
</BottomSheet>
