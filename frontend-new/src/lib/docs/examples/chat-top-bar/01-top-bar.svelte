<!-- Top bar: open the menu and pick Rename to rename in place. -->
<script lang="ts">
	import ChatTopBar from '$lib/ds/thread/ChatTopBar.svelte';

	let title = $state('Starting a new conversation');
	let renaming = $state(false);
	let pinned = $state(false);
</script>

<div class="w-full max-w-240 border-b border-border px-6 py-2.75">
	<ChatTopBar
		{title}
		meta="chat_7f3a2c"
		{pinned}
		{renaming}
		onrename={(name) => {
			title = name;
			renaming = false;
		}}
		oncancelrename={() => (renaming = false)}
		onpick={(action) => {
			if (action === 'rename') renaming = true;
			if (action === 'pin') pinned = !pinned;
		}}
		onshare={() => {}}
	/>
</div>
