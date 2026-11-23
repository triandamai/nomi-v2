<script lang="ts" module>
	/** What a file dragged out of the tree carries: its path, for a drop target (the chat) to quote. */
	export const FILE_DRAG_TYPE = 'application/x-nomi-file';
</script>

<script lang="ts">
	import { tick } from 'svelte';
	import { SvelteSet } from 'svelte/reactivity';
	import { Copy, Database, FileCode, FileText, Folder, FolderOpen, Lock, MessageSquare, Plus, TextSelect } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import CodeToken from '../chat/CodeToken.svelte';
	import MenuPanel, { type MenuItem } from '../overlays/MenuPanel.svelte';
	import type { CodeLine } from '../chat/code';

	// Build → Code view: the project's files and one file, read-only: what Koda wrote and when.
	// Pick line numbers (Shift for a range) to ask Koda about them; changes go through the chat.
	// Beyond the pen: folders open and close (click, or the arrow keys as a tree does), files drag
	// out of the tree (onto the chat, to ask about them), and a line's "+" in its gutter, shown on
	// hover or focus, opens the line's menu (Overlays / Menu): ask Koda about it, copy it, pick it.
	type Entry = { name: string; depth: number; kind: 'folder' | 'file' | 'database' | 'text'; active?: boolean; path?: string };
	type Props = {
		/** Where the files are from, e.g. "main@9d2e41a". */
		branch?: string;
		tree: Entry[];
		path: string;
		/** e.g. "read-only · edited by Koda 08:41". */
		meta?: string;
		lines: CodeLine[];
		/** Lines picked to start with, first and last. */
		selection?: [number, number];
		onopen?: (index: number) => void;
		onask: () => void;
		onasklines: (from: number, to: number) => void;
	};
	let { branch, tree, path, meta, lines, selection, onopen, onask, onasklines }: Props = $props();

	const ICONS = { folder: Folder, file: FileCode, database: Database, text: FileText } as const;
	const INDENT = ['pl-2', 'pl-5.5', 'pl-9', 'pl-12.5', 'pl-16'] as const;
	const initialRange = () => selection;
	let range = $state<[number, number] | undefined>(initialRange());
	let anchor = $state<number | undefined>(initialRange()?.[0]);
	let focused = $state(0);
	let items: HTMLElement[] = $state([]);

	// Folders open by default; a closed one hides everything below it.
	const keyOf = (entry: Entry, index: number) => entry.path ?? `${index}:${entry.name}`;
	const closed = new SvelteSet<string>();
	const shown = $derived.by(() => {
		const out: { entry: Entry; index: number; open: boolean }[] = [];
		let hiddenBelow = Infinity;
		tree.forEach((entry, index) => {
			if (entry.depth > hiddenBelow) return;
			hiddenBelow = Infinity;
			const open = entry.kind === 'folder' && !closed.has(keyOf(entry, index));
			if (entry.kind === 'folder' && !open) hiddenBelow = entry.depth;
			out.push({ entry, index, open });
		});
		return out;
	});
	function toggle(entry: Entry, index: number, open?: boolean) {
		const key = keyOf(entry, index);
		if (open ?? closed.has(key)) closed.delete(key);
		else closed.add(key);
	}
	function choose(entry: Entry, index: number) {
		if (entry.kind === 'folder') toggle(entry, index);
		else onopen?.(index);
	}
	function ondragstart(event: DragEvent, entry: Entry) {
		if (!entry.path || !event.dataTransfer) return;
		event.dataTransfer.effectAllowed = 'copy';
		event.dataTransfer.setData(FILE_DRAG_TYPE, entry.path);
		event.dataTransfer.setData('text/plain', entry.path);
	}

	// The line menu off a gutter's "+": one panel, floated in the top layer by the line it's for.
	const MENU_GAP = 4;
	const MENU_EDGE = 8;
	let menuLine = $state<number | null>(null);
	let menuAt = $state({ top: 0, left: 0 });
	let menu: HTMLElement | undefined = $state();
	let opener: HTMLElement | undefined;
	const lineText = (n: number) => (lines[n - 1] ?? []).map((token) => token.text).join('');
	// Each item keeps the line it was made for: picking one closes the menu (and clears `menuLine`) first.
	const lineItems = $derived.by<MenuItem[]>(() => {
		const n = menuLine;
		if (n === null) return [];
		return [
			{ label: m.code_line_ask({ name: 'Koda' }), icon: MessageSquare, onselect: () => onasklines(n, n) },
			{ label: m.code_line_copy(), icon: Copy, onselect: () => void navigator.clipboard?.writeText(lineText(n)).catch(() => undefined) },
			{ label: m.code_line_select(), icon: TextSelect, onselect: () => pick(n, false) },
		];
	});
	async function openMenu(event: MouseEvent, n: number) {
		opener = event.currentTarget as HTMLElement;
		const box = opener.getBoundingClientRect();
		menuLine = n;
		await tick();
		menu?.showPopover();
		const height = menu?.offsetHeight ?? 0;
		const width = menu?.offsetWidth ?? 0;
		menuAt = {
			top: box.bottom + MENU_GAP + height <= window.innerHeight ? box.bottom + MENU_GAP : Math.max(MENU_EDGE, box.top - MENU_GAP - height),
			left: Math.min(Math.max(MENU_EDGE, box.left), window.innerWidth - width - MENU_EDGE),
		};
		menu?.querySelector<HTMLElement>('[role="menuitem"]')?.focus();
	}
	function closeMenu(returnFocus: boolean) {
		menuLine = null;
		if (returnFocus) opener?.focus();
	}
	function pickFromMenu(item: MenuItem) {
		const run = item.onselect;
		closeMenu(true);
		run?.();
	}
	function onmenukey(event: KeyboardEvent) {
		const all = [...(menu?.querySelectorAll<HTMLElement>('[role="menuitem"]') ?? [])];
		const at = all.indexOf(document.activeElement as HTMLElement);
		const next = { ArrowDown: at + 1, ArrowUp: at - 1, Home: 0, End: all.length - 1 }[event.key];
		if (next !== undefined) {
			event.preventDefault();
			all[(next + all.length) % all.length]?.focus();
		} else if (event.key === 'Escape') {
			event.preventDefault();
			closeMenu(true);
		} else if (event.key === 'Tab') closeMenu(false);
	}
	$effect(() => {
		if (menuLine === null) return;
		const away = (event: PointerEvent) => {
			if (!menu?.contains(event.target as Node) && !opener?.contains(event.target as Node)) closeMenu(false);
		};
		document.addEventListener('pointerdown', away);
		return () => document.removeEventListener('pointerdown', away);
	});

	function pick(line: number, extend: boolean) {
		if (extend && anchor !== undefined) range = [Math.min(anchor, line), Math.max(anchor, line)];
		else {
			anchor = line;
			range = [line, line];
		}
	}

	const selected = (line: number) => !!range && line >= range[0] && line <= range[1];

	// `at` is the row's place among the shown rows; arrows move between those, Right/Left open and close.
	function onkeydown(event: KeyboardEvent, at: number) {
		const row = shown[at];
		const next = event.key === 'ArrowDown' ? at + 1 : event.key === 'ArrowUp' ? at - 1 : -1;
		if (next >= 0 && next < shown.length) {
			event.preventDefault();
			focused = next;
			items[next]?.focus();
		} else if (event.key === 'Enter') choose(row.entry, row.index);
		else if (row.entry.kind === 'folder' && (event.key === 'ArrowRight' || event.key === 'ArrowLeft')) {
			event.preventDefault();
			toggle(row.entry, row.index, event.key === 'ArrowRight');
		}
	}
</script>

<div class="flex h-full min-h-150 w-full overflow-hidden rounded-sm bg-bg outline -outline-offset-1 outline-border-strong">
	<div class="flex w-58 shrink-0 flex-col gap-0.5 border-r border-border bg-surface py-3 pr-1.75 pl-2 max-md:hidden">
		<p class="pb-2 font-mono text-label text-text-3"><span class="uppercase">{m.files_title()}</span>{#if branch}&nbsp;·&nbsp;{branch}{/if}</p>
		<div role="tree" aria-label={m.files_title()} class="flex flex-col gap-0.5">
			{#each shown as { entry, index, open }, at (keyOf(entry, index))}
				<div
					bind:this={items[at]}
					role="treeitem"
					aria-level={entry.depth + 1}
					aria-selected={entry.active ?? false}
					aria-expanded={entry.kind === 'folder' ? open : undefined}
					tabindex={at === focused ? 0 : -1}
					draggable={entry.kind !== 'folder' && Boolean(entry.path)}
					class={['flex h-6.5 cursor-pointer items-center gap-1.5 rounded-sm pr-2 outline-offset-0', INDENT[Math.min(entry.depth, INDENT.length - 1)], entry.active ? 'bg-surface-2' : 'hover:bg-surface-2']}
					onclick={() => choose(entry, index)}
					onkeydown={(event) => onkeydown(event, at)}
					ondragstart={(event) => ondragstart(event, entry)}
				>
					<span class={['inline-flex shrink-0', entry.active ? 'text-text' : 'text-text-3']}><Icon icon={entry.kind === 'folder' && open ? FolderOpen : ICONS[entry.kind]} size={13} /></span>
					<span class={['truncate font-mono text-caption', entry.active ? 'text-text' : 'text-text-2']}>{entry.name}</span>
				</div>
			{/each}
		</div>
	</div>
	<div class="flex min-w-0 flex-1 flex-col">
		<header class="flex h-10 shrink-0 items-center gap-2.5 border-b border-border px-4">
			<span class="inline-flex text-text-2"><Icon icon={FileCode} size={14} /></span>
			<span class="truncate font-mono text-caption text-text">{path}</span>
			<span class="min-w-0 flex-1 truncate font-mono text-caption text-text-3">{meta}</span>
			<button type="button" class="inline-flex h-6.5 shrink-0 cursor-pointer items-center gap-1.5 rounded-sm border border-border-strong px-2.25 text-caption text-text outline-offset-0 hover:bg-surface-2 focus-visible:outline-3 pointer-coarse:min-h-11" onclick={onask}>
				<Icon icon={MessageSquare} size={14} />{m.code_ask_file({ name: 'Koda' })}
			</button>
		</header>
		<div class="relative min-h-0 flex-1 overflow-auto py-3">
			<code class="flex min-w-max flex-col font-mono text-caption text-text">
				{#each lines as line, index (index)}
					{@const n = index + 1}
					<span data-line class={['group relative flex h-5.5 items-center', selected(n) && 'bg-accent-bg']}>
						<button
							type="button"
							aria-label={m.code_line_menu({ n: String(n) })}
							aria-haspopup="menu"
							aria-expanded={menuLine === n}
							class={['absolute top-0.75 left-0.5 inline-flex size-4 cursor-pointer items-center justify-center rounded-xs bg-surface-2 text-text-2 outline-offset-0 group-hover:opacity-100 hover:text-text focus-visible:opacity-100', menuLine === n ? 'opacity-100' : 'opacity-0']}
							onclick={(event) => void openMenu(event, n)}><Icon icon={Plus} size={12} /></button
						>
						<button
							type="button"
							aria-label={m.code_line({ n: String(n) })}
							aria-pressed={selected(n)}
							class={['w-11 shrink-0 cursor-pointer text-right outline-offset-0 select-none', selected(n) ? 'text-accent' : 'text-text-3 hover:text-text-2']}
							onclick={(event) => pick(n, event.shiftKey)}>{n}</button
						>
						<span class="w-4 shrink-0"></span>
						<span class="pr-4 whitespace-pre">{#each line as token, part (part)}<CodeToken {token} />{/each}</span>
					</span>
					{#if range && n === range[1]}
						<span class="flex h-0 justify-center overflow-visible">
							<button
								type="button"
								class="relative top-1 z-10 inline-flex h-7 cursor-pointer items-center gap-1.5 rounded-sm bg-text px-2.5 font-sans text-caption font-medium whitespace-nowrap text-bg shadow-float outline-offset-2"
								onclick={() => range && onasklines(range[0], range[1])}
							>
								<Icon icon={MessageSquare} size={13} />{m.code_ask_lines({ name: 'Koda', range: range[0] === range[1] ? String(range[0]) : `${range[0]}–${range[1]}` })}
							</button>
						</span>
					{/if}
				{/each}
			</code>
		</div>
		{#if menuLine !== null}
			<MenuPanel bind:element={menu} items={lineItems} onpick={pickFromMenu} popover="manual" onkeydown={onmenukey} aria-label={m.code_line({ n: String(menuLine) })} style="top: {menuAt.top}px; left: {menuAt.left}px" class="fixed m-0" />
		{/if}
		<footer class="flex h-8.5 shrink-0 items-center gap-2 border-t border-border px-4 text-caption text-text-3">
			<Icon icon={Lock} size={12} />{m.code_readonly({ name: 'Koda' })}
		</footer>
	</div>
</div>
