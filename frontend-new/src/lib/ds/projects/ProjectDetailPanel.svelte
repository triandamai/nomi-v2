<script lang="ts" module>
	export type ProjectParts = { code: string; url: string; data: string; deploys: string; members: string };
	export type ProjectChat = { title: string; age: string; href: string };
</script>

<script lang="ts">
	import type { Snippet } from 'svelte';
	import { ArrowRight, ArrowUpRight, Database, Download, GitBranch, Globe, Plus, Rocket, Users, X } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import SectionLabel from '../status/SectionLabel.svelte';
	import type { CrewId } from '../agents/crew';

	// Design System / Projects → Project / Detail Panel (I1bGE): the selected project beside the
	// grid. 380 wide on the page colour (border inside), in three parts under lines drawn inside:
	//  · its tile (32), name and live line, a 28 close; a 132 tall picture of the app (radius 6, a
	//    strong stroke over its edge); Open workspace (inverse, the rest of the row) and Visit, both
	//    32 tall (taller than Button, as the pen draws them);
	//  · what's in it: 31 rows (a 14px icon, a 72 key, the value in mono, Export on the code);
	//  · its chats (Koda's mascot, the title, the age; 36 rows under a stroke) and Ask Koda.
	type Props = {
		name: string;
		tone: CrewId;
		/** "live · deploy #14 · 2d ago": after the live dot when it's live. */
		live?: string;
		/** A picture of the app. */
		preview: Snippet;
		workspaceHref: string;
		/** The live address, to visit. */
		url?: string;
		parts: ProjectParts;
		onexport?: () => void;
		chats: ProjectChat[];
		onask: () => void;
		onclose: () => void;
	};
	let { name, tone, live, preview, workspaceHref, url, parts, onexport, chats, onask, onclose }: Props = $props();

	const rows = $derived([
		{ icon: GitBranch, key: m.projects_part_code(), value: parts.code, code: true },
		{ icon: Globe, key: m.projects_part_url(), value: parts.url, code: false },
		{ icon: Database, key: m.projects_part_data(), value: parts.data, code: false },
		{ icon: Rocket, key: m.projects_part_deploys(), value: parts.deploys, code: false },
		{ icon: Users, key: m.projects_part_members(), value: parts.members, code: false },
	]);
</script>

<aside aria-label={name} class="flex w-95 max-w-full flex-col rounded-md bg-bg outline -outline-offset-1 outline-border">
	<div class="flex flex-col gap-3.5 border-b border-border px-5 pt-5 pb-3.75">
		<div class="flex items-center gap-3">
			<span class="inline-flex size-8 shrink-0 items-center justify-center rounded-sm" style="background: var(--crew-{tone}-bg)">
				<span aria-hidden="true" class="font-app text-body font-semibold" style="color: var(--crew-{tone}-ink)">{name.charAt(0).toUpperCase()}</span>
			</span>
			<div class="flex min-w-0 flex-1 flex-col gap-0.5">
				<h2 class="truncate text-ui font-medium text-text">{name}</h2>
				{#if live}
					<p class="flex min-w-0 items-center gap-1.5">
						<span class="size-1.5 shrink-0 rounded-full bg-accent"></span>
						<span class="truncate font-mono text-label text-text-3">{live}</span>
					</p>
				{/if}
			</div>
			<button
				type="button"
				aria-label={m.action_close()}
				onclick={onclose}
				class="inline-flex size-7 shrink-0 cursor-pointer items-center justify-center rounded-sm text-text-3 outline-offset-1 hover:bg-surface-2 hover:text-text pointer-coarse:size-11"
			>
				<Icon icon={X} size={16} />
			</button>
		</div>
		<div role="img" aria-label={m.projects_detail_preview({ name })} class="h-33 overflow-hidden rounded-sm outline -outline-offset-1 outline-border-strong">{@render preview()}</div>
		<div class="flex gap-2">
			<a
				href={workspaceHref}
				class="inline-flex h-8 min-w-0 flex-1 items-center justify-center gap-1.5 rounded-sm bg-text text-ui font-medium text-bg outline-offset-0 focus-visible:outline-3 pointer-coarse:min-h-11"
			>
				{m.projects_menu_open()}
				<Icon icon={ArrowRight} size={14} />
			</a>
			{#if url}
				<a
					href="https://{url}"
					target="_blank"
					rel="noopener noreferrer"
					class="inline-flex h-8 shrink-0 items-center gap-1.5 rounded-sm border border-border-strong px-2.75 text-ui text-text outline-offset-0 hover:bg-surface-2 focus-visible:border-transparent focus-visible:outline-3 pointer-coarse:min-h-11"
				>
					{m.projects_detail_visit()}
					<span class="inline-flex text-text-2"><Icon icon={ArrowUpRight} size={14} /></span>
				</a>
			{/if}
		</div>
	</div>
	<div class="flex flex-col px-5 pt-4 pb-2">
		<SectionLabel label={m.projects_detail_parts()} />
		<dl class="mt-3 flex flex-col gap-0.5">
			{#each rows as row (row.key)}
				<div class="flex items-center gap-2.5 py-1.75">
					<span class="inline-flex text-text-3"><Icon icon={row.icon} size={14} /></span>
					<dt class="w-18 shrink-0 text-ui text-text-2">{row.key}</dt>
					<dd class="min-w-0 flex-1 truncate font-mono text-label text-text">{row.value}</dd>
					{#if row.code && onexport}
						<button type="button" onclick={onexport} class="inline-flex shrink-0 cursor-pointer items-center gap-1 rounded-xs text-caption text-text-2 outline-offset-2 hover:text-text">
							<Icon icon={Download} size={12} />
							{m.projects_detail_export()}
						</button>
					{/if}
				</div>
			{/each}
		</dl>
	</div>
	<div class="flex flex-col gap-2 border-t border-border px-5 pt-3.75 pb-4">
		<p class="flex items-center justify-between font-mono text-label text-text-3">
			<span class="tracking-label uppercase">{m.projects_detail_chats()}</span>
			<span>{chats.length}</span>
		</p>
		{#each chats as chat (chat.href)}
			<a href={chat.href} class="flex items-center gap-2.5 rounded-sm border border-border px-2.25 py-1.75 outline-offset-1 hover:bg-surface">
				<Mascot agent="koda" size={20} />
				<span class="min-w-0 flex-1 truncate text-ui text-text">{chat.title}</span>
				<span class="shrink-0 font-mono text-label text-text-3">{chat.age}</span>
			</a>
		{/each}
		<button type="button" onclick={onask} class="inline-flex cursor-pointer items-center gap-1.5 self-start rounded-xs py-1 text-ui text-text-2 outline-offset-2 hover:text-text">
			<Icon icon={Plus} size={14} />
			{m.projects_detail_ask()}
		</button>
	</div>
</aside>
