<script lang="ts">
	import { Database, FileCode, FileText, Folder, Lock, MessageSquare } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import CodeToken from '../chat/CodeToken.svelte';
	import type { CodeLine } from '../chat/code';

	// Build → Code view: the project's files and one file, read-only: what Koda wrote and when.
	// Pick line numbers (Shift for a range) to ask Koda about them; changes go through the chat.
	type Entry = { name: string; depth: number; kind: 'folder' | 'file' | 'database' | 'text'; active?: boolean };
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

	function pick(line: number, extend: boolean) {
		if (extend && anchor !== undefined) range = [Math.min(anchor, line), Math.max(anchor, line)];
		else {
			anchor = line;
			range = [line, line];
		}
	}

	const selected = (line: number) => !!range && line >= range[0] && line <= range[1];

	function onkeydown(event: KeyboardEvent, index: number) {
		const next = event.key === 'ArrowDown' ? index + 1 : event.key === 'ArrowUp' ? index - 1 : -1;
		if (next >= 0 && next < tree.length) {
			event.preventDefault();
			focused = next;
			items[next]?.focus();
		} else if (event.key === 'Enter') onopen?.(index);
	}
</script>

<div class="flex h-150 w-full overflow-hidden rounded-sm border border-border-strong bg-bg">
	<div class="flex w-58 shrink-0 flex-col gap-0.5 border-r border-border bg-surface px-2 py-3 max-md:hidden">
		<p class="pb-1.5 font-mono text-label text-text-3"><span class="tracking-label uppercase">{m.files_title()}</span>{#if branch}&nbsp;·&nbsp;{branch}{/if}</p>
		<div role="tree" aria-label={m.files_title()} class="flex flex-col gap-0.5">
			{#each tree as entry, index (index)}
				<div
					bind:this={items[index]}
					role="treeitem"
					aria-level={entry.depth + 1}
					aria-selected={entry.active ?? false}
					tabindex={index === focused ? 0 : -1}
					class={['flex h-6.5 cursor-pointer items-center gap-1.5 rounded-sm pr-2 outline-offset-0', INDENT[Math.min(entry.depth, INDENT.length - 1)], entry.active ? 'bg-surface-2' : 'hover:bg-surface-2']}
					onclick={() => onopen?.(index)}
					onkeydown={(event) => onkeydown(event, index)}
				>
					<span class={['inline-flex shrink-0', entry.active ? 'text-text' : 'text-text-3']}><Icon icon={ICONS[entry.kind]} size={13} /></span>
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
			<button type="button" class="inline-flex h-6.5 shrink-0 cursor-pointer items-center gap-1.5 rounded-sm border border-border-strong px-2.5 text-caption text-text outline-offset-0 hover:bg-surface-2 focus-visible:outline-3 pointer-coarse:min-h-11" onclick={onask}>
				<Icon icon={MessageSquare} size={14} />{m.code_ask_file({ name: 'Koda' })}
			</button>
		</header>
		<div class="relative min-h-0 flex-1 overflow-auto py-3">
			<code class="flex min-w-max flex-col font-mono text-caption text-text">
				{#each lines as line, index (index)}
					{@const n = index + 1}
					<span data-line class={['flex h-5.5 items-center', selected(n) && 'bg-accent-bg']}>
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
		<footer class="flex h-8.5 shrink-0 items-center gap-2 border-t border-border px-4 text-caption text-text-3">
			<Icon icon={Lock} size={12} />{m.code_readonly({ name: 'Koda' })}
		</footer>
	</div>
</div>
