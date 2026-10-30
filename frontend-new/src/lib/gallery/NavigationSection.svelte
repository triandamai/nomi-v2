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
</script>

{#snippet caption(text: string)}<span class="font-mono text-label text-text-3">{text}</span>{/snippet}

{#snippet nav()}
	<div class="flex flex-wrap items-start gap-6">
		<div class="flex w-56 flex-col gap-2"><NavItem href="#chats" icon={MessageSquare} label="Chats" count={12} />{@render caption('Nav Item / Default')}</div>
		<div class="flex w-56 flex-col gap-2"><NavItem href="#projects" icon={Folder} label="Projects" count={4} active />{@render caption('Nav Item / Active')}</div>
		<div class="flex w-56 flex-col gap-2"><NavGroupLabel label="Build" />{@render caption('Nav Group Label')}</div>
		<div class="flex w-56 flex-col gap-2"><RecentRow href="#chat" kind="chat" title="Budget belanja Oktober" when="2h" />{@render caption('Recent / Chat')}</div>
		<div class="flex w-56 flex-col gap-2"><RecentRow href="#project" kind="project" title="salah-daily-tracker" when="2d" />{@render caption('Recent / Project')}</div>
	</div>
{/snippet}

{#snippet identity()}
	<div class="flex flex-wrap items-start gap-8">
		<div class="flex flex-col items-start gap-2"><Avatar name="Mario Prasetyo" />{@render caption('Avatar / User')}</div>
		<div class="flex flex-col items-start gap-2">
			<span class="flex items-end gap-2"><Mascot agent="nomi" size={20} /><Mascot agent="nomi" size={32} /><Mascot agent="nomi" size={96} /></span>
			{@render caption('Mascot 20 / 32 / 96')}
		</div>
		<div class="flex flex-col items-start gap-2"><AgentByline agent="koda" name="Koda" meta="15:37 · 4m 12s" />{@render caption('Agent Byline')}</div>
		<div class="flex w-75 flex-col gap-2"><CrewRow agent="dana" name="Dana" role="Transactions, budgets, subscriptions" state="idle" />{@render caption('Crew Row')}</div>
	</div>
{/snippet}

{#snippet status()}
	<div class="flex flex-wrap items-start gap-8">
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
		<div class="flex w-50 flex-col gap-2"><Meter label="Free plan" value={0.02} caption="21.8K / 1M tokens this month" />{@render caption('Meter')}</div>
	</div>
{/snippet}

{#snippet board(title: string, note: string, content: import('svelte').Snippet)}
	<div class="flex flex-col gap-4 rounded-md border border-border p-5">
		<p class="text-ui font-medium text-text">{title} <span class="font-mono text-label text-text-3">{note}</span></p>
		{@render content()}
	</div>
{/snippet}

<GallerySection
	title="Navigation, identity & status"
	description="Active = surface-2 fill + primary text. Status is a 6px dot plus a word; colour never carries meaning alone."
>
	{@render board('Nav', 'sidebar parts · 224 wide', nav)}
	{@render board('Identity', 'people and agents', identity)}
	{@render board('Status', 'states, not decoration', status)}
</GallerySection>
