<script lang="ts">
	import { Ellipsis, Share } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import IconButton from '../actions/IconButton.svelte';
	import TopBar from '../chrome/TopBar.svelte';
	import ChatMenu from '../chat/ChatMenu.svelte';
	import RenameField from '../chat/RenameField.svelte';

	// Flow / Chat — Manage chat & long threads (J3ks0), steps 1–2: a chat's top bar. Its name, then
	// where it is in mono (the project it's filed under, or its id), and at the end Share and "···"
	// (the Chat Menu: rename, pin, move, export, share, delete), 30px ghost buttons 4 apart. Renaming
	// turns the name into the Rename Field in place: Enter saves, Escape cancels. It fills the app
	// shell's top bar, like TopBar.
	type Action = 'rename' | 'pin' | 'move' | 'export' | 'share' | 'delete';
	type Props = {
		title: string;
		meta?: string;
		pinned?: boolean;
		renaming?: boolean;
		onrename: (name: string) => void;
		oncancelrename: () => void;
		onpick: (action: Action) => void;
		onshare?: () => void;
	};
	let { title, meta, pinned = false, renaming = false, onrename, oncancelrename, onpick, onshare }: Props = $props();
</script>

{#snippet actions()}
	<span class="flex items-center gap-1">
		{#if onshare}<IconButton icon={Share} label={m.chat_share()} variant="ghost" onclick={onshare} />{/if}
		<ChatMenu {pinned} {onpick}>
			{#snippet trigger(props)}<IconButton icon={Ellipsis} label={m.chat_options()} variant="ghost" {...props} />{/snippet}
		</ChatMenu>
	</span>
{/snippet}

{#if renaming}
	<div class="flex min-w-0 items-center justify-between gap-3">
		<RenameField value={title} onsave={onrename} oncancel={oncancelrename} />
		{@render actions()}
	</div>
{:else}
	<TopBar {title} {meta} {actions} />
{/if}
