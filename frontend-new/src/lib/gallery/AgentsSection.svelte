<script lang="ts">
	import { Folder, House, Inbox, MessageSquare, Wallet } from '@lucide/svelte';
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import { CREW } from '$lib/ds/agents/crew';
	import Sidebar from '$lib/ds/chrome/Sidebar.svelte';
	import SettingsNav from '$lib/ds/chrome/SettingsNav.svelte';
	import WorkspaceChip from '$lib/ds/chrome/WorkspaceChip.svelte';
	import RecentsList from '$lib/ds/chrome/RecentsList.svelte';
	import UsageCard from '$lib/ds/chrome/UsageCard.svelte';
	import UserRow from '$lib/ds/chrome/UserRow.svelte';
	import NavItem from '$lib/ds/navigation/NavItem.svelte';
	import NavGroupLabel from '$lib/ds/navigation/NavGroupLabel.svelte';
	import GallerySection from './GallerySection.svelte';
</script>

<GallerySection
	title="Agents & app chrome"
	description="The mascot set (96 / 32 / 20) and the Sidebar every app-shell screen uses, with the Settings nav. One mascot per crew member, flat colour, used only where that member is speaking or working."
>
	<div class="flex flex-wrap gap-6">
		<div class="h-215 overflow-hidden rounded-md border border-border">
			<Sidebar newChatHref="#new" oncollapse={() => {}}>
				<NavItem href="#home" icon={House} label="Home" />
				<NavItem href="#inbox" icon={Inbox} label="Inbox" count={3} />
				<NavGroupLabel label="Talk" />
				<NavItem href="#chats" icon={MessageSquare} label="Chats" count={12} />
				<div class="flex items-center justify-between pr-2.5"><NavGroupLabel label="Build" /><WorkspaceChip name="Trian's space" /></div>
				<NavItem href="#projects" icon={Folder} label="Projects" count={4} />
				<NavGroupLabel label="Life" />
				<NavItem href="#money" icon={Wallet} label="Money" />
				{#snippet recents()}
					<RecentsList
						allHref="#chats"
						items={[
							{ href: '#r1', kind: 'chat', title: 'Starting a new conversation', when: 'now' },
							{ href: '#r2', kind: 'project', title: 'salah-daily-tracker', when: '2d' },
							{ href: '#r3', kind: 'chat', title: 'Budget belanja Oktober', when: '2h' },
							{ href: '#r4', kind: 'chat', title: 'Invoice PT Sinar Jaya', when: '3d' },
						]}
					/>
				{/snippet}
				{#snippet footer()}
					<UsageCard plan="Free plan" used={0.02} caption="21.8K / 1M tokens this month" href="#plan" />
					<UserRow name="Mario Prasetyo" settingsHref="#settings" />
				{/snippet}
			</Sidebar>
		</div>
		<div class="h-215 overflow-hidden rounded-md border border-border">
			<SettingsNav
				backHref="#home"
				active="#plan"
				groups={[
					{ label: 'Account', items: [{ href: '#profile', icon: 'user', label: 'Profile' }, { href: '#general', icon: 'settings-2', label: 'General' }, { href: '#plan', icon: 'credit-card', label: 'Plan & usage', count: 'Free' }] },
					{ label: 'AI', items: [{ href: '#crew', icon: 'users', label: 'Crew', count: 10 }, { href: '#models', icon: 'cpu', label: 'Models & providers', count: 2 }, { href: '#memory', icon: 'brain', label: 'Memory & context' }] },
					{ label: 'Workspace', items: [{ href: '#workspaces', icon: 'building-2', label: 'Workspaces', count: 2 }, { href: '#connections', icon: 'plug', label: 'Connections', count: 3 }] },
					{ label: 'Data', items: [{ href: '#privacy', icon: 'shield', label: 'Data & privacy' }] },
				]}
				user={{ name: 'Mario Prasetyo', detail: 'Free plan · Jakarta' }}
				version="v0.0.75"
				onlogout={() => {}}
			/>
		</div>
		<div class="flex min-w-0 flex-1 flex-wrap content-start gap-6">
			{#each CREW as member (member.id)}
				<div class="flex w-24 shrink-0 flex-col items-start gap-2">
					<Mascot agent={member.id} />
					<span class="flex items-end gap-2">
						<Mascot agent={member.id} size={32} />
						<Mascot agent={member.id} size={20} />
					</span>
					<span class="text-ui font-medium text-text">{member.name}</span>
					<span class="font-mono text-label text-text-3">crew-{member.id}</span>
				</div>
			{/each}
		</div>
	</div>
</GallerySection>
