<script lang="ts" module>
	export type ProjectMember = {
		initials: string;
		/** Said on hover and to screen readers. */
		name: string;
		/** The pen's three avatar fills: teal, orange, or quiet. */
		tone: 'teal' | 'orange' | 'plain';
	};
</script>

<script lang="ts">
	import { ChevronsUpDown, UserPlus } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import Button from '../actions/Button.svelte';
	import WorkspaceSwitcher, { type Workspace } from '../chrome/WorkspaceSwitcher.svelte';

	// Design System / Projects → Project / Top Bar (FRyfm): 52 tall, 24 in, its line drawn inside
	// along the bottom. The workspace crumb (a 28 button: its tile, its name, ↕) opens the workspace
	// switcher (HF1zn), then "/" and the page; at the end who is in the workspace (24px avatars,
	// 6 overlapping, a 2px ring of the page), how many, and Invite.
	type Props = {
		workspaces: Workspace[];
		onchoose: (id: string) => void;
		oncreate: () => void;
		settingsHref?: string;
		/** The page after the crumb. */
		title?: string;
		members: ProjectMember[];
		/** Everyone in the workspace, when more are in it than avatars shown. */
		memberCount?: number;
		oninvite?: () => void;
	};
	let { workspaces, onchoose, oncreate, settingsHref, title = m.projects_title(), members, memberCount, oninvite }: Props = $props();

	const chosen = $derived(workspaces.find((workspace) => workspace.chosen) ?? workspaces[0]);
	const count = $derived(memberCount ?? members.length);
</script>

<header class="relative flex h-13 items-center justify-between gap-4 px-6 after:absolute after:inset-x-0 after:bottom-0 after:h-px after:bg-border">
	<div class="flex min-w-0 items-center gap-2.5">
		<WorkspaceSwitcher current={chosen?.name ?? ''} {workspaces} {onchoose} {oncreate} {settingsHref}>
			{#snippet trigger(props)}
				<button
					type="button"
					{...props}
					aria-label={m.projects_workspace({ name: chosen?.name ?? '' })}
					class="inline-flex h-7 max-w-60 cursor-pointer items-center gap-2 rounded-sm border border-border-strong pr-1.75 pl-0.75 outline-offset-1 hover:bg-surface-2 aria-expanded:bg-surface-2 pointer-coarse:min-h-11"
				>
					<span
						aria-hidden="true"
						class={[
							'inline-flex size-5 shrink-0 items-center justify-center rounded-sm font-mono text-label font-semibold',
							chosen?.tone === 'teal' && 'bg-tile-teal text-tile-ink',
							chosen?.tone === 'amber' && 'bg-tile-amber text-tile-ink',
							chosen?.tone === 'plain' && 'bg-surface-2 text-text-2',
						]}>{chosen?.name.charAt(0).toUpperCase()}</span
					>
					<span class="truncate text-ui text-text">{chosen?.name}</span>
					<span class="inline-flex text-text-3"><Icon icon={ChevronsUpDown} size={12} /></span>
				</button>
			{/snippet}
		</WorkspaceSwitcher>
		<span aria-hidden="true" class="font-mono text-label text-text-3">/</span>
		<span class="truncate text-ui font-medium text-text">{title}</span>
	</div>
	<div class="flex shrink-0 items-center gap-2.5">
		<span role="group" aria-label={m.projects_people()} class="flex items-center">
			{#each members as member, index (index)}
				<span
					title={member.name}
					class={[
						'inline-flex size-6 items-center justify-center rounded-full border-2 border-bg text-nano font-semibold',
						index > 0 && '-ml-1.5',
						member.tone === 'teal' && 'bg-tile-teal text-tile-ink',
						member.tone === 'orange' && 'bg-tile-orange text-tile-ink',
						member.tone === 'plain' && 'bg-surface-2 text-text-2',
					]}>{member.initials}</span
				>
			{/each}
		</span>
		<span class="font-mono text-label text-text-2">{count === 1 ? m.projects_members_one() : m.projects_members({ count })}</span>
		{#if oninvite}<Button variant="secondary" label={m.projects_invite()} iconEnd={UserPlus} onclick={oninvite} />{/if}
	</div>
</header>
