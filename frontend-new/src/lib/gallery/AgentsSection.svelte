<script lang="ts">
	import { Folder, House, Inbox, MessageSquare, Wallet } from '@lucide/svelte';
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import { CREW, type CrewId } from '$lib/ds/agents/crew';
	import Sidebar from '$lib/ds/chrome/Sidebar.svelte';
	import SettingsNav from '$lib/ds/chrome/SettingsNav.svelte';
	import WorkspaceChip from '$lib/ds/chrome/WorkspaceChip.svelte';
	import RecentsList from '$lib/ds/chrome/RecentsList.svelte';
	import UsageCard from '$lib/ds/chrome/UsageCard.svelte';
	import UserRow from '$lib/ds/chrome/UserRow.svelte';
	import NavItem from '$lib/ds/navigation/NavItem.svelte';
	import NavGroupLabel from '$lib/ds/navigation/NavGroupLabel.svelte';
	import GallerySection from './GallerySection.svelte';

	// The pen's "Components / Crew Mascots" board has its own order.
	const order: CrewId[] = ['nomi', 'dana', 'kala', 'maya', 'tara', 'raga', 'personality', 'supervisor', 'rena', 'koda'];
	const mascots = order.map((id) => CREW.find((member) => member.id === id)!);
</script>

<GallerySection
	title="Agents & app chrome"
	description="The mascot set (96 / 32 / 20) and the Sidebar every app-shell screen uses, with the Settings nav. One mascot per crew member, flat colour, used only where that member is speaking or working."
>
	<!--
		The pen's Library Row: Sidebar, the Crew mascots board, Settings nav. The board's 908px basis keeps
		it beside the sidebar in the gallery's narrower column (the nav then wraps below it) and puts all
		three in one row at the pen's 1472.
	-->
	<div class="flex flex-wrap items-start gap-6">
		<div data-library="sidebar" class="h-215 shrink-0 overflow-hidden rounded-md border border-border">
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
		<div data-library="mascots" data-mascot-board class="flex min-w-0 grow basis-227 flex-col gap-7 rounded-md border border-border bg-bg p-10">
			<p class="flex flex-wrap items-end justify-between gap-x-6 gap-y-1">
				<span class="text-body font-medium text-text">Crew mascots</span>
				<span class="font-mono text-label text-text-3">1 colour + 1 silhouette per agent · flat · no head accessories</span>
			</p>
			<div class="flex flex-col gap-7 overflow-x-auto">
				<!-- Ten 96px cells spread edge to edge; the pen lets the cells overlap a little, as here. -->
				<div data-size="96" class="grid min-w-204 grid-cols-10 px-2">
					{#each mascots as member (member.id)}
						<div class="flex flex-col items-center gap-3">
							<Mascot agent={member.id} />
							<span class="text-ui whitespace-nowrap text-text-2">{member.name}</span>
						</div>
					{/each}
				</div>
				<div data-size="32" class="flex items-center gap-4">
					<span class="w-16 shrink-0 font-mono text-label text-text-3">32px</span>
					{#each mascots as member (member.id)}<Mascot agent={member.id} size={32} />{/each}
				</div>
				<div data-size="20" class="flex items-center gap-4">
					<span class="w-16 shrink-0 font-mono text-label text-text-3">20px</span>
					{#each mascots as member (member.id)}<Mascot agent={member.id} size={20} />{/each}
				</div>
			</div>
		</div>
		<div data-library="settings" class="h-215 shrink-0 overflow-hidden rounded-md border border-border">
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
	</div>
</GallerySection>
