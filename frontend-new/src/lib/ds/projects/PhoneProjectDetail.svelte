<script lang="ts">
	import type { Component, Snippet } from 'svelte';
	import { ArrowRight, ArrowUpRight, ChevronLeft, ChevronRight, Database, Download, Ellipsis, GitBranch, Globe, Rocket, Users, type LucideProps } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Mascot from '../agents/Mascot.svelte';
	import PhonePill from '../actions/PhonePill.svelte';
	import type { CrewId } from '../agents/crew';
	import type { ProjectChat, ProjectParts } from './ProjectDetailPanel.svelte';

	// Mobile / Back Bar and Mobile / Project Detail (Flow / Mobile Projects — Detail panel): a project
	// on a phone, its own screen. The back bar (48: "‹ Projects" at 16, More at the end), then
	// [8,16,24,16], 24 apart: the hero (a 56 tile with 14 corners and the initial at 24, the name at
	// 20 / 600 and the status line at 13 after its dot, the state as drawn (AL711)); what broke, where something did (danger-bg,
	// 12 corners, 14 in: the title at 15 / 600, the reason in mono at 12, the one action as an inverse
	// pill); Open workspace (inverse, the rest of the row) and Visit, 46 tall with 12 corners; WHAT'S
	// IN THIS PROJECT in 52 rows (an 18 icon, a 76 key at 15, the value in mono at 12, Export on the
	// code), and CHATS IN THIS PROJECT (Koda's mascot, the title at 15, its age, a chevron). `ask` is
	// the ask bar at the screen's foot (PhoneAskBar). The pen's picture of the app's own data is left out (§9).
	type Props = {
		name: string;
		tone: CrewId;
		/** The state, after its dot: "live · deploy #14 · 2d ago", "deploy #6 failed · #5 still live". */
		status: string;
		/** Accent when live, danger when something broke, warning while Koda is at work. */
		dot: 'live' | 'failed' | 'working' | 'idle';
		/** What broke, with the one thing to do about it. */
		alert?: { title: string; detail?: string; action?: { label: string; icon?: Component<LucideProps>; onclick: () => void } };
		workspaceHref: string;
		/** The live address, to visit. */
		url?: string;
		parts: ProjectParts;
		onexport?: () => void;
		chats: ProjectChat[];
		backLabel: string;
		onback: () => void;
		onmore?: () => void;
		ask?: Snippet;
	};
	let { name, tone, status, dot, alert, workspaceHref, url, parts, onexport, chats, backLabel, onback, onmore, ask }: Props = $props();

	const rows = $derived([
		{ icon: GitBranch, key: m.projects_part_code(), value: parts.code, code: true },
		{ icon: Globe, key: m.projects_part_url(), value: parts.url, code: false },
		{ icon: Database, key: m.projects_part_data(), value: parts.data, code: false },
		{ icon: Rocket, key: m.projects_part_deploys(), value: parts.deploys, code: false },
		{ icon: Users, key: m.projects_part_members(), value: parts.members, code: false },
	]);
</script>

<section aria-label={name} class="flex h-full flex-col bg-bg">
	<div class="flex h-12 shrink-0 items-center gap-1 px-2">
		<button type="button" class="inline-flex h-11 cursor-pointer items-center gap-0.5 pr-2 pl-1 text-phone text-text outline-offset-1" onclick={onback}>
			<ChevronLeft size={24} strokeWidth={1.5} aria-hidden="true" />{backLabel}
		</button>
		<span class="flex-1"></span>
		{#if onmore}
			<button type="button" aria-label={m.projects_actions({ name })} class="inline-flex size-11 cursor-pointer items-center justify-center rounded-full text-text outline-offset-1" onclick={onmore}>
				<Icon icon={Ellipsis} size={22} />
			</button>
		{/if}
	</div>
	<div class="min-h-0 flex-1 overflow-y-auto">
		<div class="flex flex-col gap-6 px-4 pt-2 pb-6">
			<div class="flex items-center gap-3.5">
				<span class="inline-flex size-14 shrink-0 items-center justify-center rounded-xl" style="background: var(--crew-{tone}-bg)">
					<span aria-hidden="true" class="font-app text-display font-semibold" style="color: var(--crew-{tone}-ink)">{name.charAt(0).toUpperCase()}</span>
				</span>
				<div class="flex min-w-0 flex-1 flex-col gap-1">
					<h1 class="truncate text-heading font-semibold tracking-wordmark text-text">{name}</h1>
					<p class="flex min-w-0 items-center gap-1.5">
						<span class={['size-1.75 shrink-0 rounded-full', dot === 'live' ? 'bg-accent' : dot === 'failed' ? 'bg-danger' : dot === 'working' ? 'bg-warning' : 'bg-text-3']}></span>
						<span class="truncate text-ui text-text-2">{status}</span>
					</p>
				</div>
			</div>
			{#if alert}
				<div role="status" class="flex flex-col items-start gap-2.5 rounded-lg bg-danger-bg p-3.5">
					<p class="text-body font-semibold text-danger">{alert.title}</p>
					{#if alert.detail}<p class="font-mono text-caption text-text-2">{alert.detail}</p>{/if}
					{#if alert.action}<PhonePill variant="inverse" label={alert.action.label} icon={alert.action.icon} onclick={alert.action.onclick} />{/if}
				</div>
			{/if}
			<div class="flex gap-2.5">
				<a href={workspaceHref} class="inline-flex h-11.5 min-w-0 flex-1 items-center justify-center gap-1.5 rounded-lg bg-text px-4 text-body font-medium text-bg outline-offset-2">
					{m.projects_menu_open()}<Icon icon={ArrowRight} size={16} />
				</a>
				{#if url}
					<a href="https://{url}" target="_blank" rel="noopener noreferrer" class="inline-flex h-11.5 shrink-0 items-center justify-center gap-1.5 rounded-lg border border-border-strong px-3.75 text-body font-medium text-text outline-offset-2">
						{m.projects_detail_visit()}<Icon icon={ArrowUpRight} size={16} />
					</a>
				{/if}
			</div>
			<section aria-labelledby="what-is-in" class="flex flex-col">
				<h2 id="what-is-in" class="font-mono text-label tracking-label text-text-3 uppercase">{m.projects_detail_parts()}</h2>
				{#each rows as row (row.key)}
					<div class="flex h-13 items-center gap-3 border-b border-border">
						<span class="inline-flex shrink-0 text-text-3"><Icon icon={row.icon} size={18} /></span>
						<span class="w-19 shrink-0 text-body text-text">{row.key}</span>
						<span class="min-w-0 flex-1 truncate font-mono text-caption text-text-2">{row.value}</span>
						{#if row.code && onexport}
							<button type="button" class="inline-flex h-8 shrink-0 cursor-pointer items-center gap-1.5 rounded-full border border-border-strong px-2.75 text-ui font-medium text-text outline-offset-2" onclick={onexport}>
								<Icon icon={Download} size={14} />{m.projects_detail_export()}
							</button>
						{/if}
					</div>
				{/each}
			</section>
			{#if chats.length}
				<section aria-labelledby="chats-in" class="flex flex-col">
					<div class="flex justify-between">
						<h2 id="chats-in" class="font-mono text-label tracking-label text-text-3 uppercase">{m.projects_detail_chats()}</h2>
						<span class="font-mono text-label text-text-3">{chats.length}</span>
					</div>
					{#each chats as chat (chat.href)}
						<a href={chat.href} class="flex h-13 items-center gap-3 border-b border-border -outline-offset-2">
							<Mascot agent="koda" size={20} />
							<span class="min-w-0 flex-1 truncate text-body text-text">{chat.title}</span>
							<span class="font-mono text-label text-text-3">{chat.age}</span>
							<span class="inline-flex text-text-3"><Icon icon={ChevronRight} size={16} /></span>
						</a>
					{/each}
				</section>
			{/if}
		</div>
	</div>
	{#if ask}<div class="shrink-0">{@render ask()}</div>{/if}
</section>
