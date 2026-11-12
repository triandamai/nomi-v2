<script lang="ts" module>
	import { ArrowRight, Copy, Download, EyeOff, FolderInput, Globe, Pencil, Settings, Share, Trash2 } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import type { MenuEntry, MenuItem } from '../overlays/MenuPanel.svelte';

	/** What can be done to a project from its card. Only what's given shows in the menu. */
	export type ProjectActions = {
		onopen?: () => void;
		onvisit?: () => void;
		onrename?: () => void;
		onduplicate?: () => void;
		onmove?: () => void;
		onshare?: () => void;
		onexport?: () => void;
		onsettings?: () => void;
		ontakedown?: () => void;
		ondelete?: () => void;
	};

	/**
	 * The card menu's items in the pen's groups (open · change · share · remove), a divider between
	 * groups that have anything in them. Visiting and taking down only apply to a live project.
	 * Rename (F2), Duplicate (⌘D) and Export repo (⌘E) show the keys ProjectCard answers to.
	 */
	export function projectMenu(actions: ProjectActions, live: boolean): MenuEntry[] {
		const item = (label: string, icon: MenuItem['icon'], onselect: (() => void) | undefined, extra: Partial<MenuItem> = {}): MenuItem | null =>
			onselect ? { label, icon, onselect, ...extra } : null;
		const groups = [
			[item(m.projects_menu_open(), ArrowRight, actions.onopen), live ? item(m.projects_menu_visit(), Globe, actions.onvisit) : null],
			[
				item(m.projects_menu_rename(), Pencil, actions.onrename, { shortcut: 'F2' }),
				item(m.projects_menu_duplicate(), Copy, actions.onduplicate, { shortcut: '⌘D' }),
				item(m.projects_menu_move(), FolderInput, actions.onmove),
			],
			[item(m.projects_menu_share(), Share, actions.onshare), item(m.projects_menu_export(), Download, actions.onexport, { shortcut: '⌘E' }), item(m.projects_menu_settings(), Settings, actions.onsettings)],
			[live ? item(m.projects_menu_take_down(), EyeOff, actions.ontakedown) : null, item(m.projects_menu_delete(), Trash2, actions.ondelete, { danger: true })],
		]
			.map((group) => group.filter((entry): entry is MenuItem => entry !== null))
			.filter((group) => group.length > 0);
		return groups.flatMap((group, index): MenuEntry[] => (index === 0 ? group : ['divider', ...group]));
	}
</script>

<script lang="ts">
	import { tick } from 'svelte';
	import { Ellipsis } from '@lucide/svelte';
	import IconButton from '../actions/IconButton.svelte';
	import ProjectMenuPanel from './ProjectMenuPanel.svelte';
	import { menuItems, nextIndex, placePanel } from '../overlays/floating';

	// Design System / Projects → a card's ⋯ (Icon Button / Ghost) and its Card Menu (uA0lh), floating
	// below the button's end in the top layer. Arrows, Home/End, Enter or Space pick; Escape, Tab or
	// a click elsewhere close it, and focus goes back to ⋯.
	type Props = { name: string; actions: ProjectActions; live?: boolean };
	let { name, actions, live = false }: Props = $props();

	const id = $props.id();
	const items = $derived(projectMenu(actions, live));
	let open = $state(false);
	let anchor: HTMLElement | undefined = $state();
	let menu: HTMLElement | undefined = $state();
	let position = $state({ top: 0, left: 0 });

	function place() {
		const box = anchor?.getBoundingClientRect();
		if (!box || !menu) return;
		position = placePanel(box, { width: menu.offsetWidth, height: menu.offsetHeight }, { width: window.innerWidth, height: window.innerHeight }, 'end');
	}

	async function show() {
		open = true;
		await tick();
		menu?.showPopover();
		place();
		menuItems(menu)[0]?.focus();
	}

	function hide(returnFocus: boolean) {
		open = false;
		if (returnFocus) anchor?.querySelector('button')?.focus();
	}

	function pick(item: MenuItem) {
		hide(true);
		item.onselect?.();
	}

	function onkeydown(event: KeyboardEvent) {
		const all = menuItems(menu);
		const next = nextIndex(event.key, all.indexOf(document.activeElement as HTMLElement), all.length);
		if (next !== null) {
			event.preventDefault();
			all[next]?.focus();
		} else if (event.key === 'Escape') {
			event.preventDefault();
			event.stopPropagation();
			hide(true);
		} else if (event.key === 'Tab') {
			hide(false);
		}
	}

	$effect(() => {
		if (!open) return;
		const away = (event: PointerEvent) => {
			const target = event.target as Node;
			if (!menu?.contains(target) && !anchor?.contains(target)) hide(false);
		};
		document.addEventListener('pointerdown', away);
		window.addEventListener('resize', place);
		return () => {
			document.removeEventListener('pointerdown', away);
			window.removeEventListener('resize', place);
		};
	});
</script>

{#if items.length}
	<span bind:this={anchor} class="inline-flex">
		<IconButton
			icon={Ellipsis}
			variant="ghost"
			label={m.projects_actions({ name })}
			aria-haspopup="menu"
			aria-expanded={open}
			aria-controls="{id}-menu"
			onclick={() => (open ? hide(true) : show())}
		/>
	</span>
	{#if open}
		<ProjectMenuPanel
			bind:element={menu}
			id="{id}-menu"
			title={name}
			{items}
			onpick={pick}
			popover="manual"
			{onkeydown}
			style="top: {position.top}px; left: {position.left}px"
			class="fixed m-0"
		/>
	{/if}
{/if}
