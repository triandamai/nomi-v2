<script lang="ts" module>
	import { AppWindow, ArrowUpRight, Copy, Download, FolderInput, Pencil, Power, Settings, Share, Trash2, type LucideProps } from '@lucide/svelte';
	import type { Component } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import type { ProjectActions } from './ProjectCardMenu.svelte';

	export type PhoneMenuRow = { label: string; icon: Component<LucideProps>; onselect: () => void; more?: boolean; danger?: boolean };

	/**
	 * The card sheet's rows in the phone pen's groups (Mobile / Project Menu Sheet, Card menu R4a7xS):
	 * open · change and share · remove, a separator between groups that have anything in them. The
	 * rows that open something further (Move, Share, Settings) end in a chevron. Visiting and taking
	 * down only apply to a live project. Only the actions given show.
	 */
	export function phoneProjectMenu(actions: ProjectActions, live: boolean): (PhoneMenuRow | 'separator')[] {
		const row = (label: string, icon: PhoneMenuRow['icon'], onselect: (() => void) | undefined, extra: Partial<PhoneMenuRow> = {}): PhoneMenuRow | null =>
			onselect ? { label, icon, onselect, ...extra } : null;
		const groups = [
			[row(m.projects_menu_open(), AppWindow, actions.onopen), live ? row(m.projects_menu_visit(), ArrowUpRight, actions.onvisit) : null],
			[
				row(m.projects_menu_rename(), Pencil, actions.onrename),
				row(m.projects_menu_duplicate(), Copy, actions.onduplicate),
				row(m.projects_menu_move(), FolderInput, actions.onmove, { more: true }),
				row(m.projects_menu_share(), Share, actions.onshare, { more: true }),
				row(m.projects_menu_export(), Download, actions.onexport),
				row(m.projects_menu_settings(), Settings, actions.onsettings, { more: true }),
			],
			[live ? row(m.projects_menu_take_down(), Power, actions.ontakedown) : null, row(m.projects_menu_delete(), Trash2, actions.ondelete, { danger: true })],
		]
			.map((group) => group.filter((entry): entry is PhoneMenuRow => entry !== null))
			.filter((group) => group.length > 0);
		return groups.flatMap((group, index): (PhoneMenuRow | 'separator')[] => (index === 0 ? group : ['separator', ...group]));
	}
</script>

<script lang="ts">
	import SheetRow from '../mobile/SheetRow.svelte';

	// Mobile / Project Menu Sheet (Flow / Mobile Projects — Card menu, M4p0u): a project's actions on
	// a phone, inside its BottomSheet. Sheet Rows (52, the icon at 20) in the phone's groups, with the
	// pen's own icons (app-window, arrow-up-right, power…) where they differ from the desktop menu's;
	// each group after the first under a separator 9 tall ([4,20]) whose line is inset 20 each side.
	// `onpick` runs before the row's action, so the page can close the sheet first.
	type Props = { actions: ProjectActions; live: boolean; onpick?: () => void };
	let { actions, live, onpick }: Props = $props();
</script>

{#each phoneProjectMenu(actions, live) as entry, index (index)}
	{#if entry === 'separator'}
		<span aria-hidden="true" class="flex px-5 py-1"><span class="h-px flex-1 bg-border"></span></span>
	{:else}
		<SheetRow
			icon={entry.icon}
			label={entry.label}
			more={entry.more}
			danger={entry.danger}
			onclick={() => {
				onpick?.();
				entry.onselect();
			}}
		/>
	{/if}
{/each}
