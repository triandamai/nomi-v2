<script lang="ts">
	import { Folder, MessageSquare } from '@lucide/svelte';
	import NavItem from '$lib/ds/navigation/NavItem.svelte';
	import NavGroupLabel from '$lib/ds/navigation/NavGroupLabel.svelte';
	import RecentRow from '$lib/ds/navigation/RecentRow.svelte';
	import Avatar from '$lib/ds/identity/Avatar.svelte';
	import AgentByline from '$lib/ds/identity/AgentByline.svelte';
	import CrewRow from '$lib/ds/identity/CrewRow.svelte';
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import Status from '$lib/ds/status/Status.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';
	import Meter from '$lib/ds/status/Meter.svelte';
	import GallerySection from './GallerySection.svelte';
	import GalleryBoard from './GalleryBoard.svelte';
</script>

{#snippet caption(text: string)}<span class="font-mono text-label text-text-3">{text}</span>{/snippet}

{#snippet nav()}
	<div class="flex flex-col gap-3">
		<div class="flex flex-col gap-2"><span class="w-56"><NavItem href="#chats" icon={MessageSquare} label="Chats" count={12} /></span>{@render caption('Nav Item / Default')}</div>
		<div class="flex flex-col gap-2"><span class="w-56"><NavItem href="#projects" icon={Folder} label="Projects" count={4} active /></span>{@render caption('Nav Item / Active')}</div>
		<div class="flex flex-col gap-2"><span class="w-56"><NavGroupLabel label="Build" /></span>{@render caption('Nav Group Label')}</div>
		<div class="flex flex-col gap-2"><span class="w-56"><RecentRow href="#chat" kind="chat" title="Budget belanja Oktober" when="2h" /></span>{@render caption('Recent / Chat')}</div>
		<div class="flex flex-col gap-2"><span class="w-56"><RecentRow href="#project" kind="project" title="salah-daily-tracker" when="2d" /></span>{@render caption('Recent / Project')}</div>
		<div class="flex flex-col gap-2">
			<span class="font-mono text-label text-text-2">Components / Sidebar · CKZsb — set the active Nav row via overrides</span>
			{@render caption('Sidebar (component)')}
		</div>
	</div>
{/snippet}

{#snippet identity()}
	<div class="flex flex-col gap-4">
		<div class="flex flex-wrap items-end gap-6">
			<div class="flex flex-col items-start gap-2"><Avatar name="Mario Prasetyo" />{@render caption('Avatar / User')}</div>
			<div class="flex flex-col items-start gap-2">
				<!-- The pen draws the 20 and 32 only, under the caption it gives the whole set. -->
				<span class="flex items-end gap-3"><Mascot agent="nomi" size={20} /><Mascot agent="nomi" size={32} /></span>
				{@render caption('Mascot 20 / 32 / 96')}
			</div>
			<div class="flex flex-col items-start gap-2"><AgentByline agent="koda" name="Koda" meta="15:37 · 4m 12s" />{@render caption('Agent Byline')}</div>
		</div>
		<div class="flex flex-col gap-2"><span class="w-75"><CrewRow agent="dana" name="Dana" role="Transactions, budgets, subscriptions" state="idle" /></span>{@render caption('Crew Row')}</div>
	</div>
{/snippet}

{#snippet status()}
	<div class="flex flex-wrap items-end gap-6">
		<div class="flex flex-col items-start gap-2"><Status kind="live" label="Live" />{@render caption('Status / Live')}</div>
		<div class="flex flex-col items-start gap-2"><Status kind="ready" label="ready" />{@render caption('Status / Ready')}</div>
		<div class="flex flex-col items-start gap-2"><Status kind="idle" label="idle" />{@render caption('Status / Idle')}</div>
		<div class="flex flex-col items-start gap-2"><Status kind="failed" label="Deploy failed" />{@render caption('Status / Failed')}</div>
		<div class="flex flex-col items-start gap-2"><Status kind="attention" label="Needs attention" />{@render caption('Status / Attention')}</div>
		<div class="flex flex-col items-start gap-2"><Status kind="working" label="Koda is building" agent="koda" />{@render caption('Status / Working')}</div>
		<div class="flex flex-col items-start gap-2"><Tag kind="no-go" label="No-go" />{@render caption('Tag / No-go')}</div>
		<div class="flex flex-col items-start gap-2"><Tag kind="go" label="Go" />{@render caption('Tag / Go')}</div>
		<div class="flex flex-col items-start gap-2"><Tag kind="neutral" label="Stby" />{@render caption('Tag / Neutral')}</div>
		<div class="flex flex-col items-start gap-2"><SectionLabel label="Recents" count={4} />{@render caption('Section Label')}</div>
		<div class="flex flex-col gap-2"><span class="w-50"><Meter label="Free plan" value={0.02} caption="21.8K / 1M tokens this month" /></span>{@render caption('Meter')}</div>
	</div>
{/snippet}

<GallerySection
	title="Navigation, identity & status"
	description="Active = surface-2 fill + primary text. Status is a 6px dot plus a word; colour never carries meaning alone."
>
	<div class="flex flex-col gap-6 lg:flex-row lg:items-start">
		<div class="lg:w-130 lg:shrink-0"><GalleryBoard title="Nav" note="sidebar parts · 224 wide" nomi>{@render nav()}</GalleryBoard></div>
		<div class="min-w-0 flex-1"><GalleryBoard title="Identity" note="people and agents" nomi>{@render identity()}</GalleryBoard></div>
	</div>
	<GalleryBoard title="Status" note="states, not decoration" nomi>{@render status()}</GalleryBoard>
</GallerySection>
