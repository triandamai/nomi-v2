<!-- Tabs: the phone's five, on Chats. -->
<script lang="ts">
	import { FolderOpen, House, Inbox, MessageCircle, User } from '@lucide/svelte';
	import TabBar from '$lib/ds/chrome/TabBar.svelte';
</script>

<div class="w-97.5 max-w-full">
	<TabBar
		label="Main"
		tabs={[
			{ href: '#home', label: 'Home', icon: House },
			{ href: '#chats', label: 'Chats', icon: MessageCircle, current: true },
			{ href: '#projects', label: 'Projects', icon: FolderOpen },
			{ href: '#inbox', label: 'Inbox', icon: Inbox },
			{ href: '#you', label: 'You', icon: User },
		]}
	/>
</div>
