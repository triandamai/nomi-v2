<script lang="ts">
	import { Download, FolderKanban, House, Hourglass, MessageSquare, Pencil, Pin, Plus, Send, Share2, Trash2, Wallet, X } from '@lucide/svelte';
	import NavGroupLabel from '$lib/components/ds/navigation/NavGroupLabel.svelte';
	import NavItem from '$lib/components/ds/navigation/NavItem.svelte';
	import RecentsList from '$lib/components/ds/chrome/RecentsList.svelte';
	import Sidebar from '$lib/components/ds/chrome/Sidebar.svelte';
	import SettingsNav from '$lib/components/ds/chrome/SettingsNav.svelte';
	import UsageCard from '$lib/components/ds/chrome/UsageCard.svelte';
	import UserRow from '$lib/components/ds/chrome/UserRow.svelte';
	import WorkspaceChip from '$lib/components/ds/chrome/WorkspaceChip.svelte';
	import EmptyState from '$lib/components/ds/lists/EmptyState.svelte';
	import ProjectRow from '$lib/components/ds/lists/ProjectRow.svelte';
	import SkeletonRow from '$lib/components/ds/lists/SkeletonRow.svelte';
	import SuggestionRow from '$lib/components/ds/lists/SuggestionRow.svelte';
	import Button from '$lib/components/ds/actions/Button.svelte';
	import IconButton from '$lib/components/ds/actions/IconButton.svelte';
	import Kbd from '$lib/components/ds/actions/Kbd.svelte';
	import KeyHint from '$lib/components/ds/actions/KeyHint.svelte';
	import Link from '$lib/components/ds/actions/Link.svelte';
	import Mascot from '$lib/components/ds/agents/Mascot.svelte';
	import { CREW } from '$lib/components/ds/agents/crew';
	import AgentByline from '$lib/components/ds/identity/AgentByline.svelte';
	import Avatar from '$lib/components/ds/identity/Avatar.svelte';
	import CrewRow from '$lib/components/ds/identity/CrewRow.svelte';
	import Meter from '$lib/components/ds/status/Meter.svelte';
	import SectionLabel from '$lib/components/ds/status/SectionLabel.svelte';
	import Status from '$lib/components/ds/status/Status.svelte';
	import Tag from '$lib/components/ds/status/Tag.svelte';
	import Checkbox from '$lib/components/ds/inputs/Checkbox.svelte';
	import Chip from '$lib/components/ds/inputs/Chip.svelte';
	import Search from '$lib/components/ds/inputs/Search.svelte';
	import Segmented from '$lib/components/ds/inputs/Segmented.svelte';
	import Tabs from '$lib/components/ds/inputs/Tabs.svelte';
	import TextField from '$lib/components/ds/inputs/TextField.svelte';
	import Toggle from '$lib/components/ds/inputs/Toggle.svelte';
	import Dialog from '$lib/components/ds/overlays/Dialog.svelte';
	import Menu from '$lib/components/ds/overlays/Menu.svelte';
	import Toast from '$lib/components/ds/overlays/Toast.svelte';
	import ToastRegion from '$lib/components/ds/overlays/ToastRegion.svelte';
	import Tooltip from '$lib/components/ds/overlays/Tooltip.svelte';
	import TooltipBubble from '$lib/components/ds/overlays/TooltipBubble.svelte';
	import { toasts } from '$lib/components/ds/overlays/toasts.svelte';

	// The design-system twins (frontend-new's components with the same props, drawn in Material 3),
	// to compare with frontend-new's /design-system. Documentation, not app text: not translated.
	let dialogOpen = $state(false);
	const menuItems = [
		{ label: 'Rename', icon: Pencil, shortcut: 'R' },
		{ label: 'Pin', icon: Pin },
		{ label: 'Share', icon: Share2 },
		'divider' as const,
		{ label: 'Delete', icon: Trash2, danger: true },
	];
</script>

<main class="gallery">
	<header>
		<h1 class="md-headline-medium">Design system twins</h1>
		<p class="lede">frontend-new's components with the same props and behaviour, drawn in Material 3.</p>
	</header>

	<section>
		<SectionLabel label="Actions" count={5} />
		<div class="row">
			<Button label="Deploy" />
			<Button label="Publish" variant="inverse" iconStart={Send} />
			<Button label="Export" variant="secondary" iconStart={Download} />
			<Button label="Cancel" variant="ghost" />
			<Button label="Stop all" variant="ghost-danger" />
			<Button label="Delete" variant="danger" iconStart={Trash2} />
			<Button label="Deploying · 00:14" loading />
			<Button label="Disabled" disabled />
		</div>
		<div class="row">
			<IconButton icon={Pencil} label="Rename" />
			<IconButton icon={X} label="Close" variant="ghost" />
			<IconButton icon={Send} label="Send" variant="send" />
			<Kbd keys="⌘K" />
			<KeyHint hints={[{ keys: '↵', action: 'send' }, { keys: '⇧↵', action: 'new line' }]} />
		</div>
		<div class="row">
			<Link href="/billing" label="Upgrade to Pro" />
			<Link href="/billing" label="Open in Sheets" quiet />
			<p>…or <Link href="/billing" label="switch to Nomi credits" variant="inline" /></p>
		</div>
	</section>

	<section>
		<SectionLabel label="Crew" count={CREW.length} />
		<div class="row">
			{#each CREW as member (member.id)}<Mascot agent={member.id} size={32} label={member.name} />{/each}
		</div>
		<div class="row">
			<AgentByline agent="koda" name="Koda" meta="15:37 · 4m 12s" />
			<Avatar name="Ada Lovelace" />
		</div>
		<div class="list">
			<CrewRow agent="dana" name="Dana" role="Money: budgets and spending" state="idle" href="/crew" />
			<CrewRow agent="kala" name="Kala" role="Reminders" state="working" />
		</div>
	</section>

	<section>
		<SectionLabel label="Status" count={4} />
		<div class="row">
			<Status kind="live" label="Live" />
			<Status kind="ready" label="ready" />
			<Status kind="idle" label="idle" />
			<Status kind="failed" label="Failed" />
			<Status kind="attention" label="Needs you" />
			<Status kind="working" label="Building" agent="koda" />
		</div>
		<div class="row">
			<Tag kind="go" label="Go" />
			<Tag kind="no-go" label="No-go" />
			<Tag kind="neutral" label="Standby" />
		</div>
		<div class="narrow">
			<Meter label="Tokens" value={0.42} caption="21.8K / 1M tokens this month" />
		</div>
	</section>

	<section>
		<SectionLabel label="Inputs" count={11} />
		<div class="narrow column">
			<TextField label="Project name" helper="Lowercase letters and dashes" placeholder="my-app" mono />
			<TextField label="Domain" value="nomi" error="That domain is taken. Try another." />
			<Search placeholder="Search chats" shortcut="⌘K" />
		</div>
		<div class="row">
			<Checkbox label="Remember this" checked />
			<Toggle label="Notifications" checked />
			<Chip kind="model" label="gpt-5.1" />
			<Chip kind="option" label="1 day" icon={Hourglass} />
			<Chip kind="agent" label="Koda" agent="koda" />
			<Chip kind="provision" label="postgres" />
		</div>
		<Segmented
			label="Filter"
			options={[
				{ value: 'all', label: 'All', count: 4 },
				{ value: 'live', label: 'Live', count: 2 },
				{ value: 'building', label: 'Building', count: 1 },
			]}
		/>
		<Tabs
			label="Views"
			tabs={[
				{ value: 'code', label: 'Code' },
				{ value: 'preview', label: 'Preview' },
				{ value: 'logs', label: 'Logs' },
			]}
		/>
	</section>

	<section>
		<SectionLabel label="Overlays" count={10} />
		<div class="row">
			<Button label="Open dialog" variant="secondary" onclick={() => (dialogOpen = true)} />
			<Menu items={menuItems}>
				{#snippet trigger(props)}
					<button class="trigger" {...props}>Chat menu</button>
				{/snippet}
			</Menu>
			<Tooltip label="New chat" shortcut="⌘N">
				{#snippet children(props)}<IconButton icon={Plus} label="New chat" {...props} />{/snippet}
			</Tooltip>
			<Button label="Show toast" variant="secondary" onclick={() => toasts.show({ kind: 'success', title: 'Saved', detail: 'chat · 15:37' })} />
		</div>
		<div class="row">
			<Menu inline items={menuItems} footer="Last edited 2m ago" />
			<TooltipBubble label="New chat" shortcut="⌘N" />
		</div>
		<div class="column">
			<Toast kind="success" title="Published" detail="v12 · 00:42" action={{ label: 'Open', onclick: () => {} }} ondismiss={() => {}} />
			<Toast kind="error" title="Deploy failed" detail="build exited 1" ondismiss={() => {}} />
			<Toast kind="info" title="Koda is still working" />
		</div>
		<Dialog inline title="Delete chat?" description="The chat and its files go. This can’t be undone.">
			{#snippet actions()}
				<Button label="Cancel" variant="ghost" />
				<Button label="Delete" variant="danger" />
			{/snippet}
		</Dialog>
		<Dialog bind:open={dialogOpen} title="Delete chat?" description="The chat and its files go. This can’t be undone.">
			{#snippet actions()}
				<Button label="Cancel" variant="ghost" onclick={() => (dialogOpen = false)} />
				<Button label="Delete" variant="danger" onclick={() => (dialogOpen = false)} />
			{/snippet}
		</Dialog>
	</section>

	<section>
		<SectionLabel label="App chrome" count={9} />
		<div class="frames">
			<div class="frame">
				<Sidebar newChatHref="/design-system" oncollapse={() => {}}>
					<NavItem href="/design-system" icon={House} label="Home" active />
					<NavItem href="/design-system" icon={MessageSquare} label="Chats" count={12} />
					<NavGroupLabel label="Build">
						<WorkspaceChip name="Trian's space" />
					</NavGroupLabel>
					<NavItem href="/design-system" icon={FolderKanban} label="Projects" />
					<NavItem href="/design-system" icon={Wallet} label="Money" />
					{#snippet recents()}
						<RecentsList
							allHref="/design-system"
							items={[
								{ href: '/design-system#a', kind: 'chat', title: 'Plan a trip to Bali', when: '2h' },
								{ href: '/design-system#b', kind: 'project', title: 'landing-page', when: '1d' },
							]}
						/>
					{/snippet}
					{#snippet footer()}
						<UsageCard plan="Free" used={0.08} caption="80K / 1M tokens" href="/design-system" />
						<UserRow name="Ada Lovelace" settingsHref="/design-system" />
					{/snippet}
				</Sidebar>
			</div>
			<div class="frame">
				<SettingsNav
					backHref="/design-system"
					active="/design-system#billing"
					user={{ name: 'Ada Lovelace', detail: 'ada@nomi.test' }}
					version="v1.4.0"
					onlogout={() => {}}
					groups={[
						{ label: 'Account', items: [{ href: '/design-system#profile', icon: 'user', label: 'Profile' }, { href: '/design-system#billing', icon: 'credit-card', label: 'Billing' }] },
						{ label: 'Workspace', items: [{ href: '/design-system#members', icon: 'users', label: 'Members', count: 3 }, { href: '/design-system#models', icon: 'cpu', label: 'Models' }] },
					]}
				/>
			</div>
		</div>
	</section>

	<section>
		<SectionLabel label="Lists" count={4} />
		<div class="column">
			<ProjectRow href="/design-system" name="landing-page" description="A launch page for the app" tone="koda" status={{ kind: 'live', label: 'Live' }} url="landing.nomi.app" updated="2h ago" />
			<SkeletonRow label="loading 4 projects…" />
			<EmptyState icon={FolderKanban} title="Projects" count={0} body="Nothing built yet. Ask Koda for an app and it shows up here." action={{ label: 'Start a project', href: '/design-system' }} />
			<SuggestionRow agent="dana" label="How much did I spend on food this month?" route="→ dana" />
		</div>
	</section>

	<ToastRegion />
</main>

<style>
	.gallery {
		display: flex;
		max-width: 1080px;
		flex-direction: column;
		gap: 48px;
		margin: 0 auto;
		padding: 48px 16px;
		color: var(--md-sys-color-on-surface);
	}
	:global(body) {
		background: var(--nomi-color-ground);
	}
	header {
		display: flex;
		flex-direction: column;
		gap: 8px;
	}
	h1 {
		margin: 0;
	}
	.lede {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	section {
		display: flex;
		flex-direction: column;
		gap: 20px;
	}
	.row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 12px;
	}
	.row p {
		margin: 0;
	}
	.column {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	.frames {
		display: flex;
		flex-wrap: wrap;
		gap: 24px;
	}
	.frame {
		height: 720px;
		max-width: 100%;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-large);
	}
	.narrow {
		max-width: 360px;
	}
	.list {
		max-width: 480px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.trigger {
		min-height: 40px;
		padding: 0 16px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-full);
		background: none;
		color: var(--md-sys-color-on-surface);
		cursor: pointer;
	}
	.trigger:hover {
		background: var(--md-sys-color-surface-container-high);
	}
	.trigger:focus-visible {
		outline: 3px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
</style>
