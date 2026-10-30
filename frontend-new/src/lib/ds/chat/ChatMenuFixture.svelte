<script lang="ts">
	import { Ellipsis } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import ChatMenu from './ChatMenu.svelte';
	import IconButton from '../actions/IconButton.svelte';

	// A ChatMenu off the top bar's "···", for its test and the gallery.
	let { onpick, pinned = false }: { onpick: (action: string) => void; pinned?: boolean } = $props();
</script>

<ChatMenu {pinned} {onpick}>
	{#snippet trigger(props)}<IconButton icon={Ellipsis} label={m.chat_options()} variant="ghost" {...props} />{/snippet}
</ChatMenu>
