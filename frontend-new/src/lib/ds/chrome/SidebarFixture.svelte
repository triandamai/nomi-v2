<script lang="ts">
	import { House, Inbox, MessageSquare, Folder, Wallet } from '@lucide/svelte';
	import Sidebar from './Sidebar.svelte';
	import NavItem from '../navigation/NavItem.svelte';
	import NavGroupLabel from '../navigation/NavGroupLabel.svelte';
	import WorkspaceChip from './WorkspaceChip.svelte';
	import RecentsList from './RecentsList.svelte';
	import UsageCard from './UsageCard.svelte';
	import UserRow from './UserRow.svelte';

	let { oncollapse, onswitchworkspace }: { oncollapse?: () => void; onswitchworkspace?: () => void } = $props();
</script>

<Sidebar newChatHref="/chat/new" {oncollapse}>
	<NavItem href="/" icon={House} label="Home" active />
	<NavItem href="/inbox" icon={Inbox} label="Inbox" count={3} />
	<NavGroupLabel label="Talk" />
	<NavItem href="/chats" icon={MessageSquare} label="Chats" count={12} />
	<NavGroupLabel label="Build"><WorkspaceChip name="Trian's space" onclick={onswitchworkspace} /></NavGroupLabel>
	<NavItem href="/projects" icon={Folder} label="Projects" count={4} />
	<NavGroupLabel label="Life" />
	<NavItem href="/money" icon={Wallet} label="Money" />
	{#snippet recents()}
		<RecentsList
			allHref="/chats"
			items={[
				{ href: '/chat/1', kind: 'chat', title: 'Starting a new conversation', when: 'now' },
				{ href: '/projects/2', kind: 'project', title: 'salah-daily-tracker', when: '2d' },
				{ href: '/chat/3', kind: 'chat', title: 'Budget belanja Oktober', when: '2h' },
			]}
		/>
	{/snippet}
	{#snippet footer()}
		<UsageCard plan="Free plan" used={0.02} caption="21.8K / 1M tokens this month" href="/settings/plan" />
		<UserRow name="Mario Prasetyo" settingsHref="/settings" />
	{/snippet}
</Sidebar>
