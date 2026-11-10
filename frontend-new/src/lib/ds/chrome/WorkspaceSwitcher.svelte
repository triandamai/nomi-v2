<script lang="ts" module>
	export type Workspace = {
		id: string;
		name: string;
		/** "Admin · 3 members · 4 projects", in mono. */
		meta: string;
		/** The initial's tile: quiet, or one of the pen's two fills. */
		tone: 'plain' | 'teal' | 'amber';
		/** The one projects are listed from: checked. */
		chosen: boolean;
		/** A small mono tag after the name ("Personal"). */
		tag?: string;
	};
</script>

<script lang="ts">
	import { tick } from 'svelte';
	import { Check, ChevronsUpDown, Lock, Plus, Settings2, Users } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import MenuRow from '../overlays/MenuRow.svelte';
	import type { MenuItem } from '../overlays/MenuPanel.svelte';
	import { menuItems, nextIndex, placePanel } from '../overlays/floating';

	// Project / Workspace Switcher (HF1zn, open on App / Projects t24cE): opens from the Build
	// group's chip, 360 wide (surface, border-strong inside, the larger float shadow, 6 in). Every
	// workspace the person is in, with its tile, role, people and projects; the one projects are
	// listed from is filled and checked. Then what a workspace shares and what stays personal (it
	// describes the menu), Create workspace, and the workspace's settings. Choosing one changes only
	// where projects are listed from and started: the backend can't switch anything else yet.
	// Left out on purpose: the pen's invited workspace with "Join" (the API doesn't list a person's
	// pending invites), and "shared crew templates" in the note (Nomi has none).
	// Its button is the WorkspaceChip's look, drawn here with the menu's state (the chip keeps the
	// old app's twin's props). `inline` shows the panel open in place (the gallery).
	type Props = {
		/** The chip's workspace. */
		current: string;
		workspaces: Workspace[];
		onchoose: (id: string) => void;
		oncreate: () => void;
		/** The chosen workspace's settings and members. */
		settingsHref?: string;
		inline?: boolean;
	};
	let { current, workspaces, onchoose, oncreate, settingsHref, inline = false }: Props = $props();

	const id = $props.id();
	let open = $state(false);
	let anchor: HTMLElement | undefined = $state();
	let menu: HTMLElement | undefined = $state();
	let position = $state({ top: 0, left: 0 });

	const create: MenuItem = $derived({ label: m.workspace_create(), icon: Plus, onselect: oncreate });

	function place() {
		const box = anchor?.getBoundingClientRect();
		if (!box || !menu) return;
		position = placePanel(box, { width: menu.offsetWidth, height: menu.offsetHeight }, { width: window.innerWidth, height: window.innerHeight });
	}

	async function show() {
		open = true;
		await tick();
		menu?.showPopover();
		place();
		const items = menuItems(menu);
		(items.find((item) => item.getAttribute('aria-checked') === 'true') ?? items[0])?.focus();
	}

	function hide(returnFocus: boolean) {
		open = false;
		if (returnFocus) anchor?.querySelector<HTMLElement>('button')?.focus();
	}

	function choose(workspace: Workspace) {
		if (!inline) hide(true);
		if (!workspace.chosen) onchoose(workspace.id);
	}

	function pick(item: MenuItem) {
		if (!inline) hide(true);
		item.onselect?.();
	}

	function onkeydown(event: KeyboardEvent) {
		const items = menuItems(menu);
		const next = nextIndex(event.key, items.indexOf(document.activeElement as HTMLElement), items.length);
		if (next !== null) {
			event.preventDefault();
			items[next]?.focus();
		} else if (event.key === 'Escape' && !inline) {
			event.preventDefault();
			hide(true);
		} else if (event.key === 'Tab' && !inline) {
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

{#snippet rule(icon: typeof Users, title: string, body: string)}
	<span class="flex gap-2">
		<span class="inline-flex text-text-2"><Icon {icon} size={14} /></span>
		<span class="flex min-w-0 flex-1 flex-col gap-px">
			<span class="text-caption font-medium text-text">{title}</span>
			<span class="text-caption text-text-3">{body}</span>
		</span>
	</span>
{/snippet}

{#snippet panel()}
	<div
		bind:this={menu}
		id="{id}-menu"
		role="menu"
		tabindex="-1"
		aria-labelledby="{id}-head"
		aria-describedby="{id}-boundary"
		popover={inline ? undefined : 'manual'}
		{onkeydown}
		style={inline ? undefined : `top: ${position.top}px; left: ${position.left}px`}
		class={['flex w-90 max-w-full flex-col gap-0.5 rounded-md bg-surface p-1.5 text-text shadow-float-lg outline -outline-offset-1 outline-border-strong', !inline && 'fixed m-0']}
	>
		<p id="{id}-head" role="presentation" class="px-2.5 pt-1.5 pb-1 font-mono text-label tracking-label text-text-3 uppercase">{m.workspace_switcher()}</p>
		{#each workspaces as workspace (workspace.id)}
			<button
				type="button"
				role="menuitemradio"
				tabindex="-1"
				aria-checked={workspace.chosen}
				onclick={() => choose(workspace)}
				class={[
					'flex w-full cursor-pointer items-center gap-2.5 rounded-sm px-2.5 py-2 text-left outline-none hover:bg-surface-2 focus-visible:bg-surface-2 pointer-coarse:min-h-11',
					workspace.chosen && 'bg-surface-2',
				]}
			>
				<span
					data-tile
					aria-hidden="true"
					class={[
						'inline-flex size-8 shrink-0 items-center justify-center rounded-sm text-ui font-semibold',
						workspace.tone === 'plain' && 'bg-surface-2 text-text-2',
						workspace.tone === 'teal' && 'bg-tile-teal text-tile-ink',
						workspace.tone === 'amber' && 'bg-tile-amber text-tile-ink',
					]}>{workspace.name.charAt(0).toUpperCase()}</span
				>
				<span class="flex min-w-0 flex-1 flex-col gap-0.5">
					<span class="flex min-w-0 items-center gap-1.5">
						<span class="truncate text-ui font-medium text-text">{workspace.name}</span>
						{#if workspace.tag}<span class="inline-flex h-4.5 shrink-0 items-center rounded-xs bg-bg px-1.25 font-mono text-label text-text-2 uppercase">{workspace.tag}</span>{/if}
					</span>
					<span class="truncate font-mono text-label text-text-3">{workspace.meta}</span>
				</span>
				{#if workspace.chosen}<span class="inline-flex text-text"><Icon icon={Check} /></span>{/if}
			</button>
		{/each}
		<hr class="border-border" />
		<div id="{id}-boundary" role="none" class="flex flex-col gap-2 rounded-sm bg-bg p-2.5">
			{@render rule(Users, m.workspace_shared_title(), m.workspace_shared_body())}
			{@render rule(Lock, m.workspace_personal_title(), m.workspace_personal_body())}
		</div>
		<hr class="border-border" />
		<MenuRow item={create} onpick={pick} />
		{#if settingsHref}
			<a
				href={settingsHref}
				role="menuitem"
				tabindex="-1"
				onclick={() => !inline && hide(false)}
				class="flex h-7.5 w-full cursor-pointer items-center gap-2 rounded-xs px-2 text-ui text-text outline-none hover:bg-surface-2 focus-visible:bg-surface-2 pointer-coarse:min-h-11"
			>
				<span class="inline-flex text-text-2"><Icon icon={Settings2} size={14} /></span>
				<span class="min-w-0 flex-1 truncate">{m.workspace_settings()}</span>
				<span class="font-mono text-label text-text-3">⌘ ⇧ ,</span>
			</a>
		{/if}
	</div>
{/snippet}

{#if inline}
	{@render panel()}
{:else}
	<span bind:this={anchor} class="inline-flex">
		<button
			type="button"
			aria-label={m.workspace_chip({ name: current })}
			aria-haspopup="menu"
			aria-expanded={open}
			aria-controls="{id}-menu"
			onclick={() => (open ? hide(true) : show())}
			class="inline-flex h-5 cursor-pointer items-center gap-1.25 rounded-xs bg-surface-2 pr-1.5 pl-0.75 outline-offset-1 hover:bg-border aria-expanded:bg-border"
		>
			<span aria-hidden="true" class="inline-flex size-3.5 items-center justify-center rounded-xs text-label font-semibold text-text">{current.charAt(0).toUpperCase()}</span>
			<span class="font-mono text-label text-text-2">{current}</span>
			<ChevronsUpDown size={10} strokeWidth={1.5} aria-hidden="true" class="text-text-3" />
		</button>
	</span>
	{#if open}{@render panel()}{/if}
{/if}
