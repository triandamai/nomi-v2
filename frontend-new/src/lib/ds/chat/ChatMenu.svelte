<script lang="ts">
	import type { ComponentProps } from 'svelte';
	import { Download, FolderInput, Pencil, Pin, PinOff, Share2, Trash2 } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Menu from '../overlays/Menu.svelte';

	// Chat responses → Chat Menu, off the top bar's "···": everything you can do to the chat itself.
	// Delete stands apart, in danger.
	type Action = 'rename' | 'pin' | 'move' | 'export' | 'share' | 'delete';
	type Props = { pinned?: boolean; onpick: (action: Action) => void; trigger?: ComponentProps<typeof Menu>['trigger']; inline?: boolean };
	let { pinned = false, onpick, trigger, inline = false }: Props = $props();
</script>

<Menu
	wide
	width={260}
	align="end"
	{trigger}
	{inline}
	items={[
		{ label: m.chat_rename(), icon: Pencil, shortcut: 'F2', onselect: () => onpick('rename') },
		{ label: pinned ? m.chat_unpin() : m.chat_pin(), icon: pinned ? PinOff : Pin, onselect: () => onpick('pin') },
		{ label: m.chat_move(), icon: FolderInput, onselect: () => onpick('move') },
		{ label: m.chat_export(), icon: Download, meta: 'md · pdf', onselect: () => onpick('export') },
		{ label: m.chat_share(), icon: Share2, onselect: () => onpick('share') },
		'divider',
		{ label: m.chat_delete(), icon: Trash2, danger: true, onselect: () => onpick('delete') },
	]}
/>
