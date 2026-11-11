<script lang="ts">
	import { MessageSquare, Pin, PinOff, Trash2 } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import IconButton from '../actions/IconButton.svelte';
	import Status from '../status/Status.svelte';

	// A chat in the Chats list, in the list style of the Inbox and Projects screens (App / Inbox
	// BFoiZ): a quiet row, 12 in, the chat's tile, its name and last line, and its time in mono;
	// pin and delete show on hover or focus (always on a touch screen). A pinned chat says so; one
	// the crew is working in shows it. The pen draws no Chats page, so this is the Inbox row's
	// shape with the chat's own parts.
	type Props = {
		href: string;
		title: string;
		preview?: string;
		time: string;
		pinned?: boolean;
		working?: boolean;
		onpin?: () => void;
		ondelete?: () => void;
	};
	let { href, title, preview, time, pinned = false, working = false, onpin, ondelete }: Props = $props();
</script>

<div class="group relative flex w-full items-center gap-3 rounded-sm px-3 py-2.5 hover:bg-surface has-focus-visible:bg-surface">
	<span class="inline-flex size-6.5 shrink-0 items-center justify-center rounded-sm bg-surface-2 text-text-2"><Icon icon={MessageSquare} size={14} /></span>
	<span class="flex min-w-0 flex-1 flex-col gap-0.5">
		<a {href} class="truncate rounded-xs text-ui font-medium text-text outline-offset-2 after:absolute after:inset-0">{title}</a>
		{#if preview}<span class="truncate text-caption text-text-3">{preview}</span>{/if}
	</span>
	{#if working}<Status kind="live" label={m.chats_working()} />{/if}
	{#if pinned}<span class="inline-flex text-text-3" title={m.chats_pinned()}><Icon icon={Pin} size={12} /><span class="sr-only">{m.chats_pinned()}</span></span>{/if}
	<span class="font-mono text-label text-text-3">{time}</span>
	{#if onpin || ondelete}
		<span class="relative z-10 flex items-center gap-0.5 opacity-0 group-hover:opacity-100 focus-within:opacity-100 pointer-coarse:opacity-100">
			{#if onpin}<IconButton icon={pinned ? PinOff : Pin} label={pinned ? m.chat_unpin() : m.chat_pin()} variant="ghost" iconSize={14} onclick={onpin} />{/if}
			{#if ondelete}<IconButton icon={Trash2} label={m.chat_delete()} variant="ghost" iconSize={14} onclick={ondelete} />{/if}
		</span>
	{/if}
</div>
