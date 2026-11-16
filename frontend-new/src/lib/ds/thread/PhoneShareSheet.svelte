<script lang="ts">
	import { Ellipsis, Link, Mail, MessageCircle, Send } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Toggle from '../inputs/Toggle.svelte';
	import BottomSheet from '../overlays/BottomSheet.svelte';
	import type { ShareSettings } from '../chat/SharePanel.svelte';

	// Flow / Mobile Chat — Share & delete → Share sheet (step 1): a read-only link from a phone. What
	// to share (This reply · Whole chat, as two halves on surface-2), what the link shows (the excerpt,
	// masked while Hide amounts and names is on), the two switches, then where to send it as 52
	// round tiles with their names under them: Copy link (accent), WhatsApp, Telegram, Mail, and More
	// (the phone's own share). When it expires sits at the foot in mono; tapping it changes it.
	type Props = {
		open?: boolean;
		excerpt: string;
		chatMessages: number;
		url: string;
		expiry: string;
		settings: ShareSettings;
		onchange: (settings: ShareSettings) => void;
		onexpiry?: () => void;
		oncopy: () => void;
		onclose?: () => void;
	};
	let { open = $bindable(false), excerpt, chatMessages, url, expiry, settings, onchange, onexpiry, oncopy, onclose }: Props = $props();

	const update = (patch: Partial<ShareSettings>) => onchange({ ...settings, ...patch });
	const text = $derived(encodeURIComponent(url));
	const canShare = $derived(typeof navigator !== 'undefined' && 'share' in navigator);
</script>

<BottomSheet bind:open title={m.share_title()} subtitle={m.share_read_only()} {onclose}>
	<div class="flex flex-col gap-3.5 px-4 pb-2">
		<div role="radiogroup" aria-label={m.share_scope()} class="flex rounded-lg bg-surface-2 p-1">
			{#each [{ value: 'reply', label: m.share_reply() }, { value: 'chat', label: `${m.share_chat()} · ${chatMessages}` }] as const as option (option.value)}
				<button
					type="button"
					role="radio"
					aria-checked={settings.scope === option.value}
					class={['h-9 flex-1 cursor-pointer rounded-inner text-phone-sm outline-offset-1', settings.scope === option.value ? 'bg-surface font-medium text-text' : 'text-text-2']}
					onclick={() => update({ scope: option.value })}>{option.label}</button
				>
			{/each}
		</div>
		<p class="line-clamp-3 rounded-xl bg-surface-2 px-3.5 py-3 text-phone-sm text-text">{excerpt}</p>
		{#each [{ key: 'mask', title: m.share_mask(), body: m.share_mask_phone() }, { key: 'anyone', title: m.share_anyone_phone(), body: m.share_anyone_help() }] as const as row (row.key)}
			<div class="flex items-center gap-3">
				<span class="flex min-w-0 flex-1 flex-col gap-0.5">
					<span class="text-phone-sm text-text">{row.title}</span>
					<span class="text-ui text-text-3">{row.body}</span>
				</span>
				<Toggle label={row.title} checked={settings[row.key]} onchange={(on) => update({ [row.key]: on })} />
			</div>
		{/each}
		<div class="flex justify-between gap-2 pt-1">
			{#snippet target(label: string, icon: typeof Link, accent: boolean)}
				<span class={['inline-flex size-13 items-center justify-center rounded-2xl', accent ? 'bg-accent text-accent-ink' : 'bg-surface-2 text-text']}><Icon {icon} size={20} /></span>
				<span class="text-caption text-text-2">{label}</span>
			{/snippet}
			<button type="button" class="flex w-15 cursor-pointer flex-col items-center gap-1.5 outline-offset-2" onclick={oncopy}>{@render target(m.share_copy(), Link, true)}</button>
			<a href="https://wa.me/?text={text}" target="_blank" rel="noopener noreferrer" class="flex w-15 flex-col items-center gap-1.5 outline-offset-2">{@render target('WhatsApp', MessageCircle, false)}</a>
			<a href="https://t.me/share/url?url={text}" target="_blank" rel="noopener noreferrer" class="flex w-15 flex-col items-center gap-1.5 outline-offset-2">{@render target('Telegram', Send, false)}</a>
			<a href="mailto:?body={text}" class="flex w-15 flex-col items-center gap-1.5 outline-offset-2">{@render target(m.share_mail(), Mail, false)}</a>
			{#if canShare}
				<button type="button" class="flex w-15 cursor-pointer flex-col items-center gap-1.5 outline-offset-2" onclick={() => navigator.share({ url }).catch(() => undefined)}>{@render target(m.mobile_more(), Ellipsis, false)}</button>
			{/if}
		</div>
		<button type="button" class="self-start font-mono text-label text-text-3 outline-offset-2" onclick={onexpiry} disabled={!onexpiry}>{expiry}</button>
	</div>
</BottomSheet>
